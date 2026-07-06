use super::action::EditAction;
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

#[cfg(test)]
mod tests {
    use super::authorize_replacement;
    use crate::text_edit::{plan_committed_tail_full_token_replacement, EditActionKind};

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
}
