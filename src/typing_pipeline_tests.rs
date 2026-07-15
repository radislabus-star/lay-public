use crate::config::{
    default_typing_assist_pipeline, default_typing_assist_rules, CorrectionSafety,
};
use crate::typing_assist_test_fixtures::{fixture_rows, parse_bool_fixture, single_fixture_row};
use crate::typing_candidate::TypingDecisionConfidence;
use crate::typing_pipeline::TypingRuleEvaluation;
use std::collections::HashSet;

use super::{explain_typing_assist_with_pipeline, select_typing_assist_with_pipeline};

#[test]
fn rule_graph_defines_every_default_rule() {
    for (rule_id, _) in default_typing_assist_rules() {
        assert!(
            crate::typing_rule_graph::find_typing_rule(rule_id).is_some(),
            "missing typing rule definition for {rule_id}"
        );
    }
}

#[test]
fn experimental_pipeline_uses_layout_candidates_for_autocorrect() {
    let pipeline = crate::typing_context::typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
        "djn ",
    );

    assert_eq!(
        select_typing_assist_with_pipeline("djn ", true, &pipeline),
        Some("вот ".to_string())
    );
}

#[test]
fn experimental_pipeline_keeps_normal_word_boundary_pairs() {
    let pipeline = crate::typing_context::typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
        "слов и ",
    );

    assert_eq!(
        select_typing_assist_with_pipeline("слов и ", true, &pipeline),
        None
    );
}

#[test]
fn rule_graph_ids_are_unique_and_registry_roundtrips() {
    let mut ids = HashSet::new();
    for rule in crate::typing_rule_graph::typing_rule_definitions() {
        assert!(ids.insert(rule.id), "duplicate typing rule id: {}", rule.id);
        let found = crate::typing_rule_graph::find_typing_rule(rule.id)
            .unwrap_or_else(|| panic!("missing registry lookup for {}", rule.id));
        assert_eq!(found.id, rule.id);
        assert_eq!(found.default_priority, rule.default_priority);
        assert_eq!(found.family, rule.family);
    }
}

#[test]
fn rule_graph_numeric_metadata_is_valid() {
    for rule in crate::typing_rule_graph::typing_rule_definitions() {
        assert!(
            rule.family_weight.is_finite() && rule.family_weight > 0.0,
            "invalid family weight for {}: {}",
            rule.id,
            rule.family_weight
        );
        if let Some(priority) = rule.default_priority {
            assert!(priority > 0, "invalid default priority for {}", rule.id);
        }
    }
}

#[test]
fn default_pipeline_matches_rule_graph_metadata() {
    let default_rules = default_typing_assist_rules();
    for rule in crate::typing_rule_graph::typing_rule_definitions() {
        let in_default = default_rules.iter().any(|(rule_id, _)| *rule_id == rule.id);
        assert_eq!(
            in_default,
            rule.default_priority.is_some(),
            "default pipeline metadata mismatch for {}",
            rule.id
        );
    }
}

#[test]
fn default_rule_priorities_are_unique_and_ordered() {
    let default_rules = default_typing_assist_rules();
    for pair in default_rules.windows(2) {
        assert!(
            pair[0].1 < pair[1].1,
            "default rule priorities must be strictly increasing: {}={} then {}={}",
            pair[0].0,
            pair[0].1,
            pair[1].0,
            pair[1].1
        );
    }
}

#[test]
fn explain_reports_chosen_candidate() {
    let pipeline = default_typing_assist_pipeline();
    let row = single_fixture_row("typing_pipeline_explain_chosen.tsv", 4);
    let allow_layout_auto = parse_bool_fixture(&row[1]);
    let explanation = explain_typing_assist_with_pipeline(&row[0], allow_layout_auto, &pipeline);

    assert_eq!(explanation.output, Some(row[2].clone()));
    let chosen = explanation.chosen.as_ref().expect("chosen candidate");
    assert_eq!(chosen.replacement, row[3]);
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
    let row = single_fixture_row("typing_pipeline_explain_margin.tsv", 3);
    let allow_layout_auto = parse_bool_fixture(&row[1]);
    let explanation = explain_typing_assist_with_pipeline(&row[0], allow_layout_auto, &pipeline);

    assert_eq!(explanation.output, Some(row[2].clone()));
    assert!(explanation.chosen.is_some());
    assert!(
        explanation.margin.is_some(),
        "explain path should expose a margin for confidence tuning"
    );
}

#[test]
fn explain_reports_disabled_rule() {
    let mut pipeline = default_typing_assist_pipeline();
    let row = single_fixture_row("typing_pipeline_disabled_rule.tsv", 3);
    let disabled_rule = &row[2];
    let rule = pipeline
        .iter_mut()
        .find(|rule| rule.id == *disabled_rule)
        .expect("disabled rule");
    rule.enabled = false;

    let explanation =
        explain_typing_assist_with_pipeline(&row[0], parse_bool_fixture(&row[1]), &pipeline);
    let evaluation = explanation
        .evaluations
        .iter()
        .find(|eval| eval.id == *disabled_rule)
        .expect("disabled rule evaluation");

    assert!(!evaluation.enabled);
    assert_eq!(
        evaluation.rejected.as_deref(),
        Some(TypingRuleEvaluation::REJECT_DISABLED)
    );
}

#[test]
fn explain_reports_unknown_configured_rule() {
    let mut pipeline = default_typing_assist_pipeline();
    let row = single_fixture_row("typing_pipeline_unknown_rule.tsv", 4);
    let priority = row[3].parse().expect("unknown rule priority");
    pipeline.push(crate::config::TypingAssistRuleConfig {
        id: row[2].clone(),
        enabled: true,
        priority,
    });

    let explanation =
        explain_typing_assist_with_pipeline(&row[0], parse_bool_fixture(&row[1]), &pipeline);
    let evaluation = explanation
        .evaluations
        .iter()
        .find(|eval| eval.id == row[2])
        .expect("unknown rule evaluation");

    assert!(evaluation.enabled);
    assert_eq!(
        evaluation.rejected.as_deref(),
        Some(TypingRuleEvaluation::REJECT_UNKNOWN_RULE)
    );
}

#[test]
fn apply_typing_assist_uses_explain_path() {
    let pipeline = default_typing_assist_pipeline();
    for row in fixture_rows("typing_pipeline_apply_explain.tsv") {
        assert_eq!(row.len(), 2, "apply/explain fixture must be TSV");
        let allow_layout_auto = parse_bool_fixture(&row[1]);
        let explanation =
            explain_typing_assist_with_pipeline(&row[0], allow_layout_auto, &pipeline);
        assert_eq!(
            select_typing_assist_with_pipeline(&row[0], allow_layout_auto, &pipeline),
            explanation.output
        );
    }
}
