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
    semantic_pending_space_from: Option<String>,
}

impl VerifiedTransitionReceipt {
    pub(super) fn is_pending_boundary_projection(&self) -> bool {
        self.semantic_pending_space_from.is_some()
            && self.operator == TransitionOperator::BoundaryShift
            && self.proof == TransitionProof::Boundary
    }
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
            semantic_pending_space_from: None,
        })
    }

    pub(super) fn matches(&self, action: &EditAction) -> bool {
        action.from_text() == self.from_text
            && action.to_text() == self.to_text
            && action.plan() == Some(&self.plan)
            && action.transition().operator() == Some(self.operator)
            && action.transition().proof() == Some(self.proof)
            && action.transition().is_verified()
            && self
                .semantic_pending_space_from
                .as_ref()
                .is_none_or(|semantic| {
                    semantic == &format!("{} ", action.from_text())
                        && action.transition().operator() == Some(TransitionOperator::BoundaryShift)
                        && action.transition().changed_tokens() == Some(2)
                        && action.to_text().ends_with(' ')
                        && super::diff_plan::replacement_plan_matches(
                            action.from_text(),
                            action.to_text(),
                            &self.plan,
                        )
                })
    }

    pub(super) fn project_pending_space(
        &self,
        action: &EditAction,
        physical_plan: &TextReplacement,
    ) -> Option<Self> {
        if !self.matches(action) || self.semantic_pending_space_from.is_some() {
            return None;
        }
        Some(Self {
            from_text: action.from_text().strip_suffix(' ')?.to_string(),
            to_text: self.to_text.clone(),
            plan: physical_plan.clone(),
            operator: self.operator,
            proof: self.proof,
            semantic_pending_space_from: Some(action.from_text().to_string()),
        })
    }

    pub(super) fn commit_pending_space(
        &self,
        action: &EditAction,
        plan: &TextReplacement,
    ) -> Option<Self> {
        if !self.matches(action) || !self.is_pending_boundary_projection() {
            return None;
        }
        Some(Self {
            from_text: self.semantic_pending_space_from.clone()?,
            to_text: self.to_text.clone(),
            plan: plan.clone(),
            operator: self.operator,
            proof: self.proof,
            semantic_pending_space_from: None,
        })
    }
}

/// An exact inverse of the recorded, sealed Space pair action. No unrelated
/// multiword undo or a target reconstructed from strings can obtain this proof.
pub fn plan_recorded_boundary_inverse(forward: &EditAction) -> Option<EditAction> {
    let from_text = forward.to_text();
    let to_text = format!("{} ", forward.from_text());
    let authority = TransitionAuthority::recorded_undo(from_text, &to_text);
    let mut action = EditAction::planned_replacement(PlannedReplacementInput {
        source: "ime-recorded-boundary-inverse",
        confidence_milli: 1_000,
        from_text,
        to_text: &to_text,
        plan: TextReplacement {
            move_left: 0,
            backspaces: u32::try_from(from_text.chars().count()).ok()?,
            insert: to_text.clone(),
            move_right: 0,
        },
        selected_source_id: Some("recorded_boundary_inverse"),
        selected_error_class: Some("boundary-shift"),
        transition: authority.transition().clone(),
    });
    if !action.validate_recorded_boundary_inverse(forward) {
        return None;
    }
    let action = seal_ready_action(action, Some(&authority));
    action.allow_apply().then_some(action)
}

/// A physical last-pair edit consumes exactly one pending ASCII Space. The
/// ordinary planner still rejects a boundary edit with changed whitespace.
pub fn plan_space_boundary_edit(
    source: &str,
    physical_from: &str,
    physical_to: &str,
    decision: &crate::input_gate::InputGateDecision,
) -> Option<EditAction> {
    if decision.trigger != crate::input_gate::InputGateTrigger::Space {
        return None;
    }
    let (original_left, original_right) = physical_from.split_once(' ')?;
    let (target_left, target_right) = physical_to.strip_suffix(' ')?.split_once(' ')?;
    if [original_left, original_right, target_left, target_right]
        .iter()
        .any(|word| !crate::word_reader::is_cyrillic_word(word))
    {
        return None;
    }
    // General form candidates stay in the shared lattice. This stricter lexical
    // evidence grants only the new final-pair pending-Space physical scope.
    let field = crate::hot_field::HotFieldSnapshot::current();
    if !field.boundary_observed_form_is_unknown(original_right)
        || crate::russian_lexicon::has_clean_russian_surface_certificate(original_right)
        || !field.boundary_form_is_attested(target_left)
        || !field.boundary_form_is_attested(target_right)
    {
        return None;
    }
    let semantic_from = format!("{physical_from} ");
    plan_input_gate_edit(
        source,
        &semantic_from,
        physical_to,
        TextReplacement {
            move_left: 0,
            backspaces: u32::try_from(semantic_from.chars().count()).ok()?,
            insert: physical_to.to_string(),
            move_right: 0,
        },
        decision,
    )
    .project_pending_boundary_space()
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
    // The executor must preserve the DecisionCore's final competition result.
    // Bayes is one input signal only; using it alone can downgrade a verified
    // BoundaryCell32 winner after the full L2/L3/L4 field already admitted it.
    let confidence_milli = trace
        .and_then(|trace| {
            trace
                .candidate_scores
                .iter()
                .find(|score| score.selected)
                .map(|score| score.decision_rank_milli)
                .or(trace.scoreboard.selected_bayes_posterior_milli)
        })
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

pub fn plan_ime_manual_toggle_edit(
    from_text: &str,
    to_text: &str,
    plan: TextReplacement,
) -> EditAction {
    plan_manual_edit(
        "ibus-active-composition-manual-toggle",
        1_000,
        from_text,
        to_text,
        plan,
        1,
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

pub fn plan_ime_candidate_accept_edit(
    source: &str,
    confidence_milli: i16,
    from_text: impl Into<String>,
    to_text: impl Into<String>,
) -> EditAction {
    let from_text = from_text.into();
    let to_text = to_text.into();
    let Some(fallback_plan) = super::diff_plan::plan_text_replacement(&from_text, &to_text) else {
        return EditAction::keep(source, from_text);
    };
    let plan = if super::transition::completion_projection_is_valid(&from_text, &to_text) {
        fallback_plan
    } else if super::transition::ime_full_token_replacement_is_valid(&from_text, &to_text) {
        TextReplacement {
            move_left: 0,
            backspaces: from_text.chars().count() as u32,
            insert: to_text.clone(),
            move_right: 0,
        }
    } else {
        fallback_plan
    };
    let authority = TransitionAuthority::ime_candidate_acceptance(&from_text, &to_text);
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
            selected_source_id: Some("ImeCandidateAcceptCell32"),
            selected_error_class: Some("ime-candidate-accept"),
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
    seal_ready_action(EditAction::planned_replacement(input), authority)
}

fn seal_ready_action(
    mut action: EditAction,
    authority: Option<&TransitionAuthority>,
) -> EditAction {
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
        TextReplacement, TransitionAudit, TransitionOperator, TransitionProof,
    };
    use crate::typing_transition::decision::DecisionTransitionReceipt;

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
    fn gate_authorizes_same_transition_behind_unchanged_right_context() {
        let receipt = DecisionTransitionReceipt::for_visible_tail(
            "постаивм ".to_string(),
            "поставим ".to_string(),
            TransitionAudit::proven(
                TransitionOperator::ReplaceCurrentWord,
                TransitionProof::Typo,
                true,
                false,
                1,
            ),
        );
        let action = super::plan_decision_transition_edit(
            crate::text_edit::DecisionTransitionEditInput {
                source: "typing-assist",
                confidence_milli: 800,
                from_text: "постаивм хвост",
                to_text: "поставим хвост",
                plan: TextReplacement {
                    move_left: 6,
                    backspaces: 8,
                    insert: "поставим".to_string(),
                    move_right: 6,
                },
                selected_source_id: Some("CanonicalL2FieldReadout"),
                selected_error_class: Some("adjacent-transposition"),
            },
            &receipt,
        );

        assert_eq!(action.kind(), EditActionKind::ReplaceRange);
        assert!(action.allow_apply(), "action={action:?}");
        assert!(action.has_verifier_receipt());
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
