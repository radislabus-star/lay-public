use super::action::EditAction;
use super::mutation::TransitionAudit;
use super::types::TextReplacement;

pub fn authorize_replacement(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    selected_source_id: Option<&str>,
    selected_error_class: Option<&str>,
) -> EditAction {
    EditAction::planned_replacement(
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id,
        selected_error_class,
    )
}

pub fn authorize_replacement_with_transition(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    selected_source_id: Option<&str>,
    selected_error_class: Option<&str>,
    transition: TransitionAudit,
) -> EditAction {
    authorize_replacement(
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id,
        selected_error_class,
    )
    .with_transition(transition)
}

#[cfg(test)]
mod tests {
    use super::{authorize_replacement, authorize_replacement_with_transition};
    use crate::text_edit::{
        plan_committed_tail_full_token_replacement, plan_text_replacement, EditActionKind,
        TransitionAudit,
    };

    #[test]
    fn gate_authorizes_last_token_replacement() {
        let plan =
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan");
        let action = authorize_replacement(
            "typing-assist",
            700,
            "провека ",
            "проверка ",
            plan,
            Some("missing-letter"),
            Some("missing-letter"),
        );

        assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
        assert!(action.allow_apply());
    }

    #[test]
    fn gate_blocks_unverified_left_context_transition() {
        let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
        let action = authorize_replacement_with_transition(
            "typing-assist",
            700,
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
    }
}
