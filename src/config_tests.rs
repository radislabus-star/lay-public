use super::*;

#[test]
fn config_defaults_preserve_public_runtime_behavior() {
    let cfg = LayConfig::default();
    assert_eq!(cfg.mode, "simple");
    assert_eq!(cfg.active_replace_words(), 1);
    assert_eq!(cfg.active_correction_engine(), CorrectionEngine::Replay);
    assert_eq!(cfg.active_text_backend(), TextBackendPreference::Uinput);
    assert!(!cfg.force_layout_hotkeys);
    assert_eq!(cfg.force_ru_key, "single-rctrl");
    assert_eq!(cfg.force_en_key, "single-ralt");
    assert!(!cfg.multi_tap_scope);
    assert!(!cfg.enter_autocorrect);
    assert_eq!(cfg.active_correction_safety(), CorrectionSafety::Normal);
    assert_eq!(cfg.active_multi_tap_max_taps(), 4);
    assert!(cfg.auto_switch_layout);
    assert!(!cfg.nanda_autocorrect);
    assert!(!cfg.nanda_trace);
    assert!(!cfg.nanda_trace_text);
    assert!(!cfg.nanda_precognition);
    assert!(!cfg.debug_action_log);
    assert!(cfg.lem_enabled);
    assert!(cfg.lem_enabled_for_scope(2));
    assert!(cfg.lem_enabled_for_scope(3));
    assert_eq!(cfg.lem_weight_percent, 80);
    assert_eq!(cfg.nanda_l2_weight_percent, 20);
    assert_eq!(cfg.nanda_l3_weight_percent, 8);
    assert!(cfg.llmwave_shadow);
    assert!(cfg.llmwave_apply);
    assert!(cfg.nanda_l2_phase_shadow);
    assert!(!cfg.nanda_l2_phase_apply);
    assert!(cfg.nanda_l3_phase_shadow);
    assert_eq!(cfg.active_lem_weight(), 1.0);
    assert_eq!(cfg.active_nanda_l2_weight(), 1.0);
    assert_eq!(cfg.active_nanda_l3_weight(), 1.0);
    assert_eq!(
        cfg.active_typing_assist_pipeline().len(),
        default_typing_assist_rules().len()
    );
}

#[test]
fn lem_master_switch_disables_all_lem_scopes_and_weight() {
    let cfg = LayConfig {
        lem_enabled: false,
        lem_2_words: true,
        lem_3_words: true,
        lem_weight_percent: 200,
        ..LayConfig::default()
    };

    assert!(!cfg.lem_enabled_for_scope(2));
    assert!(!cfg.lem_enabled_for_scope(3));
    assert_eq!(cfg.active_lem_weight(), 0.0);
}

#[test]
fn influence_weights_are_clamped_to_safe_range() {
    let cfg = LayConfig {
        lem_weight_percent: 250,
        nanda_l2_weight_percent: 201,
        nanda_l3_weight_percent: 0,
        ..LayConfig::default()
    };

    assert_eq!(cfg.active_lem_weight(), 2.5);
    assert_eq!(cfg.active_nanda_l2_weight(), 10.0);
    assert_eq!(cfg.active_nanda_l3_weight(), 0.0);
}

#[test]
fn nanda_precognition_requires_ime_backend_and_positive_nanda_weight() {
    let disabled_weights = LayConfig {
        text_backend: "ime".to_string(),
        nanda_precognition: true,
        nanda_l2_weight_percent: 0,
        nanda_l3_weight_percent: 0,
        ..LayConfig::default()
    };
    assert!(!disabled_weights.active_nanda_precognition());

    let l2_enabled = LayConfig {
        nanda_l2_weight_percent: 1,
        ..disabled_weights.clone()
    };
    assert!(l2_enabled.active_nanda_precognition());

    let uinput_backend = LayConfig {
        text_backend: "uinput".to_string(),
        ..l2_enabled
    };
    assert!(!uinput_backend.active_nanda_precognition());

    let auto_backend = LayConfig {
        text_backend: "auto".to_string(),
        nanda_l2_weight_percent: 1,
        ..disabled_weights
    };
    assert!(auto_backend.active_nanda_precognition());
}

#[test]
fn legacy_config_without_force_hotkeys_gets_safe_defaults() {
    let cfg: LayConfig =
        serde_json::from_str(r#"{"mode":"simple","trigger":"double-lshift"}"#).unwrap();

    assert!(!cfg.force_layout_hotkeys);
    assert_eq!(cfg.force_ru_key, "single-rctrl");
    assert_eq!(cfg.force_en_key, "single-ralt");
    assert!(!cfg.multi_tap_scope);
    assert_eq!(cfg.active_multi_tap_max_taps(), 4);
}

#[test]
fn multi_tap_max_taps_is_clamped_to_runtime_range() {
    let too_low = LayConfig {
        multi_tap_max_taps: 1,
        ..LayConfig::default()
    };
    let too_high = LayConfig {
        multi_tap_max_taps: 9,
        ..LayConfig::default()
    };

    assert_eq!(too_low.active_multi_tap_max_taps(), 2);
    assert_eq!(too_high.active_multi_tap_max_taps(), 4);
}

#[test]
fn legacy_llm_mode_maps_to_smart_only_without_explicit_engine() {
    let legacy = LayConfig {
        mode: "llm".into(),
        ..LayConfig::default()
    };
    let explicit_replay = LayConfig {
        mode: "llm".into(),
        correction_engine: Some("replay".into()),
        ..LayConfig::default()
    };

    assert_eq!(legacy.active_correction_engine(), CorrectionEngine::Smart);
    assert_eq!(
        explicit_replay.active_correction_engine(),
        CorrectionEngine::Replay
    );
}

#[test]
fn auto_replace_off_keeps_layout_only_rules() {
    let pipeline =
        typing_assist_pipeline_for_auto_replace(false, &default_typing_assist_pipeline());
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "missing_letter")
        .is_some_and(|rule| !rule.enabled));
}

#[test]
fn auto_replace_on_disables_risky_deletion_rules() {
    let pipeline = typing_assist_pipeline_for_auto_replace(true, &default_typing_assist_pipeline());
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "repeated_letter")
        .is_some_and(|rule| rule.enabled));
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "layout_ru_to_en")
        .is_some_and(|rule| rule.enabled));
    let risky = "layout_en_to_ru";
    assert!(
        pipeline
            .iter()
            .find(|rule| rule.id == risky)
            .is_some_and(|rule| !rule.enabled),
        "{risky} must not run in normal live autocorrect"
    );
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "extra_letters")
        .is_some_and(|rule| !rule.enabled));
    for risky in [
        "single_letter_substitution",
        "verb_ending",
        "vowel_confusion",
    ] {
        assert!(
            pipeline
                .iter()
                .find(|rule| rule.id == risky)
                .is_some_and(|rule| !rule.enabled),
            "{risky} must stay experimental for live autocorrect"
        );
    }
}

#[test]
fn correction_safety_controls_typing_assist_risk() {
    let strict = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Strict,
        &default_typing_assist_pipeline(),
    );
    assert!(strict
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));
    assert!(strict
        .iter()
        .find(|rule| rule.id == "missing_letter")
        .is_some_and(|rule| !rule.enabled));

    let experimental = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
    );
    assert!(experimental
        .iter()
        .find(|rule| rule.id == "extra_letters")
        .is_some_and(|rule| rule.enabled));
}

#[test]
fn correction_safety_requirement_matrix_is_explicit() {
    use crate::typing_rule_graph::TypingRuleRequiredSafety;

    assert!(
        CorrectionSafety::Strict.allows_typing_rule_requirement(TypingRuleRequiredSafety::Strict)
    );
    assert!(
        !CorrectionSafety::Strict.allows_typing_rule_requirement(TypingRuleRequiredSafety::Normal)
    );
    assert!(!CorrectionSafety::Normal
        .allows_typing_rule_requirement(TypingRuleRequiredSafety::Experimental));
    assert!(CorrectionSafety::Experimental
        .allows_typing_rule_requirement(TypingRuleRequiredSafety::Experimental));
}
