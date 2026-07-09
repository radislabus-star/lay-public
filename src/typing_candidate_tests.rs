use super::{
    choose_typing_candidate, classify_typing_confidence, classify_typing_rule,
    rank_typing_candidates, score_typing_candidate, TypingCandidate, TypingCandidateFamily,
    TypingDecisionConfidence,
};
use crate::typing_assist_test_fixtures::fixture_rows;

fn fixture_candidates(name: &str) -> Vec<TypingCandidate> {
    fixture_rows(name)
        .into_iter()
        .map(|row| {
            assert_eq!(row.len(), 4, "typing candidate fixture must be TSV");
            let priority = row[1].parse().expect("candidate priority");
            TypingCandidate::new(&row[0], priority, &row[2], row[3].clone())
        })
        .collect()
}

#[test]
fn classifies_rules_without_runtime_word_lists() {
    assert_eq!(
        classify_typing_rule("personal_token"),
        TypingCandidateFamily::Exact
    );
    assert_eq!(
        classify_typing_rule("layout_ru_to_en"),
        TypingCandidateFamily::Layout
    );
    assert_eq!(
        classify_typing_rule("glued_phrase"),
        TypingCandidateFamily::Structural
    );
    assert_eq!(
        classify_typing_rule("missing_letter"),
        TypingCandidateFamily::Typo
    );
}

#[test]
fn exact_rule_beats_generic_typo_candidate() {
    let chosen = choose_typing_candidate(fixture_candidates("typing_candidate_exact_vs_typo.tsv"))
        .expect("candidate");

    assert_eq!(chosen.rule_id, "personal_token");
    assert_eq!(chosen.replacement, "примерно");
}

#[test]
fn typo_candidate_can_beat_structural_space_repair() {
    let chosen = choose_typing_candidate(fixture_candidates(
        "typing_candidate_typo_vs_structural.tsv",
    ))
    .expect("candidate");

    assert_eq!(chosen.rule_id, "missing_letter");
}

#[test]
fn pure_space_repair_scores_above_non_pure_split() {
    let candidates = fixture_candidates("typing_candidate_pure_space.tsv");
    let pure = &candidates[0];
    let noisy = &candidates[1];

    assert!(pure.score.structure_bonus > noisy.score.structure_bonus);
}

#[test]
fn score_total_is_named_component_sum() {
    let score = score_typing_candidate("словослитно ", "слово слитно ", "glued_phrase", 100);

    let expected = score.family_weight
        + score.language_delta
        + score.structure_bonus
        + score.lexical_prior_bonus
        + score.priority_bonus
        - score.edit_penalty
        - score.intervention_penalty
        - score.weak_grammar_penalty;
    assert!((score.total - expected).abs() < f64::EPSILON);
    assert!(score.structure_bonus > 0.0);
    assert!(score.lexical_prior_bonus >= 0.0);
    assert!(score.weak_grammar_penalty >= 0.0);
    assert!(score.edit_penalty >= 0.0);
    assert!(score.intervention_penalty >= 0.0);
}

#[test]
fn experimental_layout_rules_score_above_normal_layout_rules() {
    let normal = score_typing_candidate("z ", "я ", "layout_en_to_ru", 100);
    let experimental = score_typing_candidate("z ", "я ", "experimental_layout_en_to_ru", 99);

    assert!(experimental.family_weight > normal.family_weight);
    assert!(experimental.total > normal.total);
}

#[test]
fn score_components_are_finite_for_edge_inputs() {
    for (original, replacement, priority) in [
        ("", "", 0),
        ("", "а", 0),
        ("word", "", -1),
        ("словослитно ", "слово слитно ", 100),
        ("ошисбя", "ошибся", 130),
        ("QR-rjlf", "QR-кода", 80),
    ] {
        let score = score_typing_candidate(original, replacement, "missing_letter", priority);
        assert!(score.total.is_finite(), "total must be finite: {score:?}");
        assert!(
            score.family_weight.is_finite(),
            "family weight must be finite: {score:?}"
        );
        assert!(
            score.language_delta.is_finite(),
            "language delta must be finite: {score:?}"
        );
        assert!(
            score.lexical_prior_bonus.is_finite(),
            "lexical prior bonus must be finite: {score:?}"
        );
        assert!(
            score.weak_grammar_penalty.is_finite(),
            "weak grammar penalty must be finite: {score:?}"
        );
        assert!(
            score.structure_bonus.is_finite(),
            "structure bonus must be finite: {score:?}"
        );
        assert!(
            score.edit_penalty.is_finite(),
            "edit penalty must be finite: {score:?}"
        );
        assert!(
            score.intervention_penalty.is_finite(),
            "intervention penalty must be finite: {score:?}"
        );
        assert!(
            score.priority_bonus.is_finite(),
            "priority bonus must be finite: {score:?}"
        );
    }
}

#[test]
fn candidate_owns_safety_lookup_for_its_rule() {
    let candidate = TypingCandidate::new("layout_en_to_ru", 100, "api", "фзш".to_string());

    assert_eq!(
        candidate.is_safe_for("api"),
        crate::typing_rule_graph::typing_rule_candidate_is_safe("layout_en_to_ru", "api", "фзш")
    );
}

#[test]
fn priority_breaks_ties_inside_same_family() {
    let chosen = choose_typing_candidate(fixture_candidates("typing_candidate_priority_tie.tsv"))
        .expect("candidate");

    assert_eq!(chosen.replacement, "wird");
}

#[test]
fn ranked_decision_keeps_second_candidate_and_margin() {
    let decision =
        rank_typing_candidates(fixture_candidates("typing_candidate_ranked_decision.tsv"))
            .expect("ranked decision");

    assert_eq!(decision.best.rule_id, "missing_letter");
    assert!(decision.second.is_some());
    assert!(decision.margin.is_finite());
    assert!(decision.is_strong(0.0));
}

#[test]
fn ranked_decision_classifies_confidence() {
    let lone = rank_typing_candidates(fixture_candidates("typing_candidate_confidence_lone.tsv"))
        .expect("lone candidate");

    assert_eq!(
        lone.confidence(1.0),
        TypingDecisionConfidence::SingleCandidate
    );

    let close = rank_typing_candidates(fixture_candidates("typing_candidate_confidence_close.tsv"))
        .expect("close candidates");

    assert_eq!(
        close.confidence(f64::INFINITY),
        TypingDecisionConfidence::Weak
    );
    assert_eq!(
        classify_typing_confidence(true, Some(2.0), 1.0),
        TypingDecisionConfidence::Strong
    );
    assert_eq!(
        classify_typing_confidence(true, None, 1.0),
        TypingDecisionConfidence::Weak
    );
}
