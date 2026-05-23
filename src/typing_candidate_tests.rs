use super::{
    choose_typing_candidate, classify_typing_confidence, classify_typing_rule,
    rank_typing_candidates, TypingCandidate, TypingCandidateFamily, TypingDecisionConfidence,
};

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
    let original = "пример";
    let chosen = choose_typing_candidate([
        TypingCandidate::new("missing_letter", 10, original, "примера".to_string()),
        TypingCandidate::new("personal_token", 100, original, "примерно".to_string()),
    ])
    .expect("candidate");

    assert_eq!(chosen.rule_id, "personal_token");
    assert_eq!(chosen.replacement, "примерно");
}

#[test]
fn typo_candidate_can_beat_structural_space_repair() {
    let original = "словослитно ";
    let chosen = choose_typing_candidate([
        TypingCandidate::new("missing_letter", 10, original, "словослитное ".to_string()),
        TypingCandidate::new("glued_phrase", 200, original, "слово слитно ".to_string()),
    ])
    .expect("candidate");

    assert_eq!(chosen.rule_id, "missing_letter");
}

#[test]
fn pure_space_repair_scores_above_non_pure_split() {
    let original = "словослитно ";
    let pure = TypingCandidate::new("glued_phrase", 100, original, "слово слитно ".to_string());
    let noisy = TypingCandidate::new("glued_phrase", 100, original, "слова слитно ".to_string());

    assert!(pure.score.structure_bonus > noisy.score.structure_bonus);
}

#[test]
fn priority_breaks_ties_inside_same_family() {
    let original = "word";
    let chosen = choose_typing_candidate([
        TypingCandidate::new("hard_sign", 200, original, "ward".to_string()),
        TypingCandidate::new("hard_sign", 10, original, "wird".to_string()),
    ])
    .expect("candidate");

    assert_eq!(chosen.replacement, "wird");
}

#[test]
fn ranked_decision_keeps_second_candidate_and_margin() {
    let original = "словослитно ";
    let decision = rank_typing_candidates([
        TypingCandidate::new("missing_letter", 10, original, "словослитное ".to_string()),
        TypingCandidate::new("glued_phrase", 200, original, "слово слитно ".to_string()),
        TypingCandidate::new("hard_sign", 300, original, "словослитноъ ".to_string()),
    ])
    .expect("ranked decision");

    assert_eq!(decision.best.rule_id, "missing_letter");
    assert!(decision.second.is_some());
    assert!(decision.margin.is_finite());
    assert!(decision.is_strong(0.0));
}

#[test]
fn ranked_decision_classifies_confidence() {
    let original = "word";
    let lone = rank_typing_candidates([TypingCandidate::new(
        "hard_sign",
        10,
        original,
        "ward".to_string(),
    )])
    .expect("lone candidate");

    assert_eq!(
        lone.confidence(1.0),
        TypingDecisionConfidence::SingleCandidate
    );

    let close = rank_typing_candidates([
        TypingCandidate::new("hard_sign", 10, original, "ward".to_string()),
        TypingCandidate::new("hard_sign", 10, original, "wbrd".to_string()),
    ])
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
