use super::action::EditAction;
use super::mutation::{TransitionAudit, TransitionOperator, TransitionProof};
use super::types::TextReplacement;

// This is the public text-edit gate boundary; keep metadata explicit at call sites.
#[allow(clippy::too_many_arguments)]
pub(crate) fn plan_verified_transition_edit(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    selected_source_id: Option<&str>,
    selected_error_class: Option<&str>,
    transition: TransitionAudit,
) -> EditAction {
    EditAction::planned_replacement(
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id,
        selected_error_class,
        transition,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_input_gate_edit(
    source: &str,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    trace: &crate::action_log::RecentActionGateTrace,
) -> EditAction {
    let confidence_milli = trace
        .scoreboard
        .as_ref()
        .and_then(|scoreboard| scoreboard.selected_bayes_posterior_milli)
        .unwrap_or(0);
    plan_verified_transition_edit(
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        trace.selected_source_id.as_deref(),
        trace.selected_error_class.as_deref(),
        trace.selected_transition_audit(),
    )
}

pub fn plan_manual_edit(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    changed_tokens: usize,
) -> EditAction {
    plan_verified_transition_edit(
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        Some("manual_toggle"),
        None,
        TransitionAudit::proven(
            TransitionOperator::ManualReplace,
            TransitionProof::ManualIntent,
            true,
            false,
            changed_tokens.max(1),
        ),
    )
}

pub fn plan_native_edit(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    changed_tokens: usize,
) -> EditAction {
    plan_verified_transition_edit(
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        Some("manual_native_replace"),
        None,
        TransitionAudit::proven(
            TransitionOperator::NativeReplace,
            TransitionProof::NativeIntent,
            true,
            false,
            changed_tokens.max(1),
        ),
    )
}

pub fn plan_recorded_undo_edit(
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    changed_tokens: usize,
) -> EditAction {
    plan_verified_transition_edit(
        "auto-undo",
        1000,
        from_text,
        to_text,
        plan,
        Some("auto_undo"),
        None,
        TransitionAudit::proven(
            TransitionOperator::Undo,
            TransitionProof::UndoRecord,
            true,
            false,
            changed_tokens.max(1),
        ),
    )
}

pub fn plan_ime_completion_edit(
    source: &str,
    confidence_milli: i16,
    from_text: impl Into<String>,
    to_text: impl Into<String>,
) -> EditAction {
    EditAction::ime_accept(source, confidence_milli, from_text.into(), to_text.into())
}

#[cfg(test)]
mod tests {
    use super::plan_verified_transition_edit;
    use crate::text_edit::{
        plan_committed_tail_full_token_replacement, plan_text_replacement, EditActionKind,
        TransitionAudit, TransitionOperator, TransitionProof,
    };

    #[test]
    fn gate_authorizes_last_token_replacement() {
        let plan =
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan");
        let action = plan_verified_transition_edit(
            "typing-assist",
            700,
            "провека ",
            "проверка ",
            plan,
            Some("missing-letter"),
            Some("missing-letter"),
            TransitionAudit::proven(
                TransitionOperator::ReplaceCurrentWord,
                TransitionProof::Typo,
                true,
                false,
                1,
            ),
        );

        assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
        assert!(action.allow_apply());
    }

    #[test]
    fn gate_blocks_unverified_left_context_transition() {
        let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
        let action = plan_verified_transition_edit(
            "typing-assist",
            700,
            "одно два ",
            "однотри ",
            plan,
            Some("nanda"),
            Some("glued-words"),
            TransitionAudit::proven(
                TransitionOperator::BoundaryMergeSplit,
                TransitionProof::Boundary,
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
