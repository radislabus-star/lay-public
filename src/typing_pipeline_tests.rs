use crate::config::{default_typing_assist_pipeline, DEFAULT_TYPING_ASSIST_RULES};
use crate::typing_candidate::TypingDecisionConfidence;

use super::{apply_typing_assist_with_pipeline, explain_typing_assist_with_pipeline};

#[test]
fn rule_graph_defines_every_default_rule() {
    for (rule_id, _) in DEFAULT_TYPING_ASSIST_RULES {
        assert!(
            crate::typing_rule_graph::find_typing_rule(rule_id).is_some(),
            "missing typing rule definition for {rule_id}"
        );
    }
}

#[test]
fn default_pipeline_mentions_every_runtime_rule() {
    for rule in crate::typing_rule_graph::typing_rule_definitions() {
        if rule.id == "contextual_layout_en_to_ru" {
            continue;
        }
        assert!(
            DEFAULT_TYPING_ASSIST_RULES
                .iter()
                .any(|(rule_id, _)| *rule_id == rule.id),
            "runtime typing rule {} is missing from default config",
            rule.id
        );
    }
}

#[test]
fn explain_reports_chosen_candidate() {
    let pipeline = default_typing_assist_pipeline();
    let explanation = explain_typing_assist_with_pipeline("кторое ", false, &pipeline);

    assert_eq!(explanation.output, Some("которое ".to_string()));
    let chosen = explanation.chosen.as_ref().expect("chosen candidate");
    assert_eq!(chosen.replacement, "которое");
    assert!(explanation.margin.is_some());
    assert_eq!(
        explanation.confidence(1.0),
        Some(TypingDecisionConfidence::SingleCandidate)
    );
    assert!(explanation.evaluations.iter().any(|eval| {
        eval.id == chosen.rule_id
            && eval.rejected.is_none()
            && eval
                .candidate
                .as_ref()
                .is_some_and(|candidate| candidate.replacement == chosen.replacement)
    }));
}

#[test]
fn explain_reports_candidate_margin() {
    let pipeline = default_typing_assist_pipeline();
    let explanation = explain_typing_assist_with_pipeline("тоесамое ", false, &pipeline);

    assert_eq!(explanation.output, Some("тоже самое ".to_string()));
    assert!(explanation.chosen.is_some());
    assert!(
        explanation.margin.is_some(),
        "explain path should expose a margin for confidence tuning"
    );
}

#[test]
fn explain_reports_disabled_rule() {
    let mut pipeline = default_typing_assist_pipeline();
    let rule = pipeline
        .iter_mut()
        .find(|rule| rule.id == "missing_letter")
        .expect("missing_letter rule");
    rule.enabled = false;

    let explanation = explain_typing_assist_with_pipeline("кторое ", false, &pipeline);
    let evaluation = explanation
        .evaluations
        .iter()
        .find(|eval| eval.id == "missing_letter")
        .expect("missing_letter evaluation");

    assert!(!evaluation.enabled);
    assert_eq!(evaluation.rejected.as_deref(), Some("disabled"));
}

#[test]
fn explain_reports_unknown_configured_rule() {
    let mut pipeline = default_typing_assist_pipeline();
    pipeline.push(crate::config::TypingAssistRuleConfig {
        id: "unknown_future_rule".to_string(),
        enabled: true,
        priority: 999,
    });

    let explanation = explain_typing_assist_with_pipeline("кторое ", false, &pipeline);
    let evaluation = explanation
        .evaluations
        .iter()
        .find(|eval| eval.id == "unknown_future_rule")
        .expect("unknown rule evaluation");

    assert!(evaluation.enabled);
    assert_eq!(evaluation.rejected.as_deref(), Some("unknown rule"));
}

#[test]
fn apply_typing_assist_uses_explain_path() {
    let pipeline = default_typing_assist_pipeline();
    for (text, allow_layout_auto) in [
        ("кторое ", false),
        ("ашду ", true),
        ("проверка ", false),
        ("тожесамое ", false),
    ] {
        let explanation = explain_typing_assist_with_pipeline(text, allow_layout_auto, &pipeline);
        assert_eq!(
            apply_typing_assist_with_pipeline(text, allow_layout_auto, &pipeline),
            explanation.output
        );
    }
}
