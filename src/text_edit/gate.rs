use super::action::{DecisionTransitionEditInput, EditAction, PlannedReplacementInput};
use super::mutation::{TransitionAudit, TransitionOperator, TransitionProof};
use super::transition::TransitionAuthority;
use super::types::TextReplacement;
use crate::typing_transition::decision::DecisionTransitionReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifiedTransitionReceipt {
    from_text: String,
    to_text: String,
    plan: TextReplacement,
    operator: TransitionOperator,
    proof: TransitionProof,
}

impl VerifiedTransitionReceipt {
    fn issue(authority: &TransitionAuthority, action: &EditAction) -> Option<Self> {
        let plan = action.plan()?.clone();
        if !authority.matches_transition(action.transition()) {
            return None;
        }
        let transition = action.transition();
        let operator = transition.operator()?;
        let proof = transition.proof()?;
        Some(Self {
            from_text: action.from_text().to_string(),
            to_text: action.to_text().to_string(),
            plan,
            operator,
            proof,
        })
    }

    pub(super) fn matches(&self, action: &EditAction) -> bool {
        action.from_text() == self.from_text
            && action.to_text() == self.to_text
            && action.plan() == Some(&self.plan)
            && action.transition().operator() == Some(self.operator)
            && action.transition().proof() == Some(self.proof)
            && action.transition().is_verified()
    }
}

pub(crate) fn plan_decision_transition_edit(
    input: DecisionTransitionEditInput<'_>,
    receipt: &DecisionTransitionReceipt,
) -> EditAction {
    let DecisionTransitionEditInput {
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id,
        selected_error_class,
    } = input;
    let authority = TransitionAuthority::automatic_decision(receipt, from_text, to_text);
    let transition = authority
        .as_ref()
        .map(|authority| authority.transition().clone())
        .unwrap_or_default();
    seal_authorized_action(
        PlannedReplacementInput {
            source,
            confidence_milli,
            from_text,
            to_text,
            plan,
            selected_source_id,
            selected_error_class,
            transition,
        },
        authority.as_ref(),
    )
}

pub fn plan_input_gate_edit(
    source: &str,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    decision: &crate::input_gate::InputGateDecision,
) -> EditAction {
    let trace = decision.trace.as_ref();
    let confidence_milli = trace
        .and_then(|trace| trace.scoreboard.selected_bayes_posterior_milli)
        .unwrap_or(0);
    let Some(receipt) = decision
        .correction
        .as_ref()
        .and_then(|resolution| resolution.selected_transition.as_ref())
    else {
        return EditAction::keep(source, from_text);
    };
    plan_decision_transition_edit(
        DecisionTransitionEditInput {
            source,
            confidence_milli,
            from_text,
            to_text,
            plan,
            selected_source_id: trace.and_then(|trace| trace.selected_source_id.as_deref()),
            selected_error_class: trace
                .and_then(|trace| trace.selected_error_class)
                .map(|error_class| error_class.as_str()),
        },
        receipt,
    )
}

pub fn plan_manual_edit(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    _changed_tokens: usize,
) -> EditAction {
    let authority = TransitionAuthority::explicit_user_intent(from_text, to_text);
    let transition = authority.transition().clone();
    seal_authorized_action(
        PlannedReplacementInput {
            source,
            confidence_milli,
            from_text,
            to_text,
            plan,
            selected_source_id: Some("manual_toggle"),
            selected_error_class: None,
            transition,
        },
        Some(&authority),
    )
}

pub fn plan_native_edit(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    _changed_tokens: usize,
) -> EditAction {
    let authority = TransitionAuthority::native_intent(from_text, to_text);
    let transition = authority.transition().clone();
    seal_authorized_action(
        PlannedReplacementInput {
            source,
            confidence_milli,
            from_text,
            to_text,
            plan,
            selected_source_id: Some("manual_native_replace"),
            selected_error_class: None,
            transition,
        },
        Some(&authority),
    )
}

pub fn plan_recorded_undo_edit(
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    _changed_tokens: usize,
) -> EditAction {
    let authority = TransitionAuthority::recorded_undo(from_text, to_text);
    let transition = authority.transition().clone();
    seal_authorized_action(
        PlannedReplacementInput {
            source: "auto-undo",
            confidence_milli: 1000,
            from_text,
            to_text,
            plan,
            selected_source_id: Some("auto_undo"),
            selected_error_class: None,
            transition,
        },
        Some(&authority),
    )
}

pub fn plan_ime_completion_edit(
    source: &str,
    confidence_milli: i16,
    from_text: impl Into<String>,
    to_text: impl Into<String>,
) -> EditAction {
    let from_text = from_text.into();
    let to_text = to_text.into();
    let Some(plan) = super::diff_plan::plan_text_replacement(&from_text, &to_text) else {
        return EditAction::keep(source, from_text);
    };
    let authority = TransitionAuthority::completion_acceptance(&from_text, &to_text);
    let transition = authority
        .as_ref()
        .map(|authority| authority.transition().clone())
        .unwrap_or_else(TransitionAudit::none);
    let mut action = seal_authorized_action(
        PlannedReplacementInput {
            source,
            confidence_milli,
            from_text: &from_text,
            to_text: &to_text,
            plan,
            selected_source_id: Some("ImeCompletionCell32"),
            selected_error_class: Some("ime-completion"),
            transition,
        },
        authority.as_ref(),
    );
    if action.allow_apply() {
        action.mark_ime_accept();
    }
    action
}

fn seal_authorized_action(
    input: PlannedReplacementInput<'_>,
    authority: Option<&TransitionAuthority>,
) -> EditAction {
    let mut action = EditAction::planned_replacement(input);
    if action.safety().is_some_and(|safety| safety.allow_apply) {
        if let Some(receipt) =
            authority.and_then(|authority| VerifiedTransitionReceipt::issue(authority, &action))
        {
            action.attach_verification(receipt);
        }
    }
    action
}

#[cfg(test)]
mod tests {
    use super::{plan_manual_edit, seal_authorized_action, PlannedReplacementInput};
    use crate::text_edit::{
        plan_committed_tail_full_token_replacement, plan_text_replacement, EditActionKind,
        TransitionAudit, TransitionOperator, TransitionProof,
    };

    #[test]
    fn gate_authorizes_last_token_replacement() {
        let plan =
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan");
        let action = plan_manual_edit("manual-test", 1000, "провека ", "проверка ", plan, 1);

        assert_eq!(action.kind(), EditActionKind::ReplaceLastToken);
        assert!(action.allow_apply());
        assert!(action.has_verifier_receipt());
    }

    #[test]
    fn gate_blocks_unverified_left_context_transition() {
        let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
        let action = seal_authorized_action(
            PlannedReplacementInput {
                source: "typing-assist",
                confidence_milli: 700,
                from_text: "одно два ",
                to_text: "однотри ",
                plan,
                selected_source_id: Some("nanda"),
                selected_error_class: Some("glued-words"),
                transition: TransitionAudit::proven(
                    TransitionOperator::BoundaryMergeSplit,
                    TransitionProof::Boundary,
                    false,
                    true,
                    2,
                ),
            },
            None,
        );

        assert_eq!(action.kind(), EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "edit_transition_not_verified");
    }

    #[test]
    fn arbitrary_verified_audit_cannot_self_seal_without_authority() {
        let plan =
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan");
        let action = seal_authorized_action(
            PlannedReplacementInput {
                source: "typing-assist",
                confidence_milli: 1000,
                from_text: "провека ",
                to_text: "проверка ",
                plan,
                selected_source_id: Some("L2SurfaceMotifCell32"),
                selected_error_class: Some("missing-letter"),
                transition: TransitionAudit::proven(
                    TransitionOperator::ReplaceCurrentWord,
                    TransitionProof::Typo,
                    true,
                    false,
                    1,
                ),
            },
            None,
        );

        assert_eq!(action.kind(), EditActionKind::ReplaceLastToken);
        assert!(!action.allow_apply());
        assert!(!action.has_verifier_receipt());
    }
}
