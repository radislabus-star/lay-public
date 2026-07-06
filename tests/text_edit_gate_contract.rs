use lay::text_edit::{
    authorize_replacement, plan_committed_tail_full_token_replacement, EditActionKind,
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
