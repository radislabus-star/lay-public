use lay::text_edit::{
    authorize_replacement, authorize_replacement_with_transition,
    plan_committed_tail_full_token_replacement, plan_text_replacement, EditActionKind,
    TransitionAudit,
};

#[test]
fn public_text_edit_gate_keeps_destructive_replacement_behind_safety() {
    let plan = plan_committed_tail_full_token_replacement("ошипка ", "ошибка ").expect("plan");
    let action = authorize_replacement(
        "contract-test",
        720,
        "ошипка ",
        "ошибка ",
        plan,
        Some("missing-letter"),
        Some("missing-letter"),
    );

    assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
    assert!(action.allow_apply());
}

#[test]
fn public_text_mutation_gate_blocks_unverified_left_context_transition() {
    let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
    let action = authorize_replacement_with_transition(
        "contract-test",
        720,
        "одно два ",
        "однотри ",
        plan,
        Some("nanda"),
        Some("glued-words"),
        TransitionAudit::proven(
            "boundary_transition",
            "left_context_changed_without_boundary_proof",
            false,
            true,
            2,
        ),
    );

    assert_eq!(action.kind, EditActionKind::BlockUnsafe);
    assert!(!action.allow_apply());
    assert_eq!(action.safety_reason(), "edit_transition_not_verified");
    assert_eq!(action.transition.changed_tokens, Some(2));
}
