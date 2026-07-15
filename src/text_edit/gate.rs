use super::action::{DecisionTransitionEditInput, EditAction, PlannedReplacementInput};
use super::mutation::{TransitionAudit, TransitionOperator, TransitionProof};
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
    fn issue(action: &EditAction) -> Option<Self> {
        let plan = action.plan()?.clone();
        let transition = action.transition();
        let operator = transition.operator()?;
        let proof = transition.proof()?;
        if !transition.is_verified() || !operator_proof_pair_is_valid(operator, proof) {
            return None;
        }
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
    let transition = receipt
        .projected_transition(from_text, to_text)
        .unwrap_or_default();
    seal_verified_action(PlannedReplacementInput {
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id,
        selected_error_class,
        transition,
    })
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
    seal_verified_action(PlannedReplacementInput {
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id: Some("manual_toggle"),
        selected_error_class: None,
        transition: TransitionAudit::proven(
            TransitionOperator::ManualReplace,
            TransitionProof::ManualIntent,
            true,
            left_context_changed(from_text, to_text),
            changed_token_count(from_text, to_text),
        ),
    })
}

pub fn plan_native_edit(
    source: &str,
    confidence_milli: i16,
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    _changed_tokens: usize,
) -> EditAction {
    seal_verified_action(PlannedReplacementInput {
        source,
        confidence_milli,
        from_text,
        to_text,
        plan,
        selected_source_id: Some("manual_native_replace"),
        selected_error_class: None,
        transition: TransitionAudit::proven(
            TransitionOperator::NativeReplace,
            TransitionProof::NativeIntent,
            true,
            left_context_changed(from_text, to_text),
            changed_token_count(from_text, to_text),
        ),
    })
}

pub fn plan_recorded_undo_edit(
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
    _changed_tokens: usize,
) -> EditAction {
    seal_verified_action(PlannedReplacementInput {
        source: "auto-undo",
        confidence_milli: 1000,
        from_text,
        to_text,
        plan,
        selected_source_id: Some("auto_undo"),
        selected_error_class: None,
        transition: TransitionAudit::proven(
            TransitionOperator::Undo,
            TransitionProof::UndoRecord,
            true,
            left_context_changed(from_text, to_text),
            changed_token_count(from_text, to_text),
        ),
    })
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
    let transition = if completion_projection_is_valid(&from_text, &to_text) {
        TransitionAudit::proven(
            TransitionOperator::Completion,
            TransitionProof::Completion,
            true,
            false,
            1,
        )
    } else {
        TransitionAudit::none()
    };
    let mut action = seal_verified_action(PlannedReplacementInput {
        source,
        confidence_milli,
        from_text: &from_text,
        to_text: &to_text,
        plan,
        selected_source_id: Some("ImeCompletionCell32"),
        selected_error_class: Some("ime-completion"),
        transition,
    });
    if action.allow_apply() {
        action.mark_ime_accept();
    }
    action
}

fn seal_verified_action(input: PlannedReplacementInput<'_>) -> EditAction {
    let mut action = EditAction::planned_replacement(input);
    if action.safety().is_some_and(|safety| safety.allow_apply) {
        if let Some(receipt) = VerifiedTransitionReceipt::issue(&action) {
            action.attach_verification(receipt);
        }
    }
    action
}

fn operator_proof_pair_is_valid(operator: TransitionOperator, proof: TransitionProof) -> bool {
    matches!(
        (operator, proof),
        (
            TransitionOperator::ReplaceCurrentWord,
            TransitionProof::Typo
                | TransitionProof::Layout
                | TransitionProof::Context
                | TransitionProof::Grammar
        ) | (
            TransitionOperator::LayoutProjection,
            TransitionProof::Layout
        ) | (
            TransitionOperator::BoundaryShift
                | TransitionOperator::BoundaryMergeSplit
                | TransitionOperator::SplitPreviousGluedAndRepairTail,
            TransitionProof::Boundary
        ) | (
            TransitionOperator::PhraseTokenRepair,
            TransitionProof::Context
        ) | (TransitionOperator::Completion, TransitionProof::Completion)
            | (
                TransitionOperator::VisibleTail,
                TransitionProof::VisibleState
            )
            | (
                TransitionOperator::DecoderTail,
                TransitionProof::DecoderPlan
            )
            | (
                TransitionOperator::ManualReplace,
                TransitionProof::ManualIntent
            )
            | (TransitionOperator::Undo, TransitionProof::UndoRecord)
            | (
                TransitionOperator::EnterAutocorrect,
                TransitionProof::Context | TransitionProof::Typo | TransitionProof::Layout
            )
            | (
                TransitionOperator::NativeReplace,
                TransitionProof::NativeIntent
            )
    )
}

fn completion_projection_is_valid(from_text: &str, to_text: &str) -> bool {
    let from = from_text.trim_end_matches(char::is_whitespace);
    let to = to_text.trim_end_matches(char::is_whitespace);
    !from.is_empty() && to.len() > from.len() && to.starts_with(from)
}

fn left_context_changed(from_text: &str, to_text: &str) -> bool {
    let from_words = crate::word_reader::normalized_text_words(from_text);
    let to_words = crate::word_reader::normalized_text_words(to_text);
    let from_prefix = from_words.get(..from_words.len().saturating_sub(1));
    let to_prefix = to_words.get(..to_words.len().saturating_sub(1));
    from_prefix != to_prefix
}

fn changed_token_count(from_text: &str, to_text: &str) -> usize {
    let from_words = crate::word_reader::normalized_text_words(from_text);
    let to_words = crate::word_reader::normalized_text_words(to_text);
    if from_words.len() != to_words.len() {
        return from_words.len().max(to_words.len()).max(1);
    }
    from_words
        .iter()
        .zip(to_words.iter())
        .filter(|(left, right)| left != right)
        .count()
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::{plan_manual_edit, seal_verified_action, PlannedReplacementInput};
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
        let action = seal_verified_action(PlannedReplacementInput {
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
        });

        assert_eq!(action.kind(), EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "edit_transition_not_verified");
    }
}
