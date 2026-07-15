use super::mutation::TransitionAudit;
use super::safety::{autocorrect_edit_safety, EditPlanSafetyReport};
use super::types::TextReplacement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditActionKind {
    Keep,
    Suggest,
    ReplaceLastToken,
    ReplaceRange,
    SplitToken,
    GlueTokens,
    AcceptImeCandidate,
    SwitchLayout,
    BlockUnsafe,
}

impl EditActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Suggest => "suggest",
            Self::ReplaceLastToken => "replace_last_token",
            Self::ReplaceRange => "replace_range",
            Self::SplitToken => "split_token",
            Self::GlueTokens => "glue_tokens",
            Self::AcceptImeCandidate => "accept_ime_candidate",
            Self::SwitchLayout => "switch_layout",
            Self::BlockUnsafe => "block_unsafe",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditAction {
    pub kind: EditActionKind,
    pub source: String,
    pub confidence_milli: i16,
    pub from_text: String,
    pub to_text: String,
    pub plan: Option<TextReplacement>,
    pub safety: Option<EditPlanSafetyReport>,
    pub transition: TransitionAudit,
    pub selected_source_id: Option<String>,
    pub selected_error_class: Option<String>,
}

impl EditAction {
    pub fn keep(source: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            kind: EditActionKind::Keep,
            source: source.into(),
            confidence_milli: 0,
            from_text: text.clone(),
            to_text: text,
            plan: None,
            safety: None,
            transition: TransitionAudit::none(),
            selected_source_id: None,
            selected_error_class: None,
        }
    }

    pub(crate) fn planned_replacement(
        source: impl Into<String>,
        confidence_milli: i16,
        from_text: impl Into<String>,
        to_text: impl Into<String>,
        plan: TextReplacement,
        selected_source_id: Option<&str>,
        selected_error_class: Option<&str>,
    ) -> Self {
        let from_text = from_text.into();
        let to_text = to_text.into();
        let mut safety = autocorrect_edit_safety(
            &from_text,
            &to_text,
            &plan,
            selected_source_id,
            selected_error_class,
        );
        downgrade_low_confidence_boundary_edit(&mut safety, confidence_milli);
        downgrade_low_confidence_wide_edit(&mut safety, confidence_milli, selected_error_class);
        let kind = classify_planned_replacement(&safety);
        Self {
            kind,
            source: source.into(),
            confidence_milli,
            from_text,
            to_text,
            plan: Some(plan),
            safety: Some(safety),
            transition: TransitionAudit::none(),
            selected_source_id: selected_source_id.map(str::to_string),
            selected_error_class: selected_error_class.map(str::to_string),
        }
    }

    pub fn ime_accept(
        source: impl Into<String>,
        confidence_milli: i16,
        from_text: impl Into<String>,
        to_text: impl Into<String>,
    ) -> Self {
        let from_text = from_text.into();
        let to_text = to_text.into();
        let plan = super::diff_plan::plan_text_replacement(&from_text, &to_text);
        let safety = plan.as_ref().map(|plan| {
            autocorrect_edit_safety(
                &from_text,
                &to_text,
                plan,
                Some("ImeCompletionCell32"),
                Some("ime-completion"),
            )
        });
        let kind = if safety.as_ref().is_some_and(|safety| !safety.allow_apply) {
            EditActionKind::BlockUnsafe
        } else {
            EditActionKind::AcceptImeCandidate
        };
        Self {
            kind,
            source: source.into(),
            confidence_milli,
            from_text,
            to_text,
            plan,
            safety,
            transition: TransitionAudit::none(),
            selected_source_id: Some("ImeCompletionCell32".to_string()),
            selected_error_class: Some("ime-completion".to_string()),
        }
        .with_transition(TransitionAudit::proven(
            "ime_active_composition_completion",
            "visible_preedit_completion_selected",
            true,
            false,
            1,
        ))
    }

    pub fn with_transition(mut self, transition: TransitionAudit) -> Self {
        if let Some(reason) = transition.block_reason() {
            self.kind = EditActionKind::BlockUnsafe;
            if let Some(safety) = self.safety.as_mut() {
                safety.allow_apply = false;
                safety.reason = reason;
            }
        }
        self.transition = transition;
        self
    }

    pub fn allow_apply(&self) -> bool {
        self.safety
            .as_ref()
            .map(|safety| safety.allow_apply)
            .unwrap_or(!matches!(self.kind, EditActionKind::BlockUnsafe))
    }

    pub fn safety_reason(&self) -> &'static str {
        self.safety
            .as_ref()
            .map(|safety| safety.reason)
            .unwrap_or("no_safety_report")
    }

    pub fn boundary_changed(&self) -> bool {
        self.safety
            .as_ref()
            .map(|safety| safety.boundary_changed)
            .unwrap_or(false)
    }

    pub fn touched_words(&self) -> usize {
        self.safety
            .as_ref()
            .map(|safety| safety.would_touch_words)
            .unwrap_or(0)
    }

    pub(crate) fn execution_rejection_reason(&self) -> Option<&'static str> {
        if !self.allow_apply() {
            return Some(self.safety_reason());
        }
        if !matches!(
            self.kind,
            EditActionKind::ReplaceLastToken
                | EditActionKind::ReplaceRange
                | EditActionKind::SplitToken
                | EditActionKind::GlueTokens
                | EditActionKind::AcceptImeCandidate
                | EditActionKind::SwitchLayout
        ) {
            return Some("non_executable_edit_action");
        }
        if self.plan.is_none() {
            return Some("verified_edit_plan_missing");
        }
        if !self.transition.is_verified() {
            return Some("verified_transition_proof_missing");
        }
        None
    }
}

fn downgrade_low_confidence_boundary_edit(
    safety: &mut EditPlanSafetyReport,
    confidence_milli: i16,
) {
    if !safety.allow_apply || !safety.boundary_changed || !safety.word_count_changed {
        return;
    }
    if confidence_milli >= 500 {
        return;
    }
    safety.allow_apply = false;
    safety.reason = "low_confidence_boundary_edit";
}

fn downgrade_low_confidence_wide_edit(
    safety: &mut EditPlanSafetyReport,
    confidence_milli: i16,
    selected_error_class: Option<&str>,
) {
    if !safety.allow_apply || !(safety.changes_non_last_word || safety.would_touch_words > 1) {
        return;
    }
    if selected_error_class == Some("boundary-shift") || confidence_milli >= 750 {
        return;
    }
    safety.allow_apply = false;
    safety.reason = "low_confidence_wide_edit";
}

fn classify_planned_replacement(safety: &EditPlanSafetyReport) -> EditActionKind {
    if !safety.allow_apply {
        return EditActionKind::BlockUnsafe;
    }
    if safety.word_count_changed && safety.inserted_contains_space {
        return EditActionKind::SplitToken;
    }
    if safety.word_count_changed {
        return EditActionKind::GlueTokens;
    }
    if safety.boundary_changed || safety.changes_non_last_word || safety.would_touch_words > 1 {
        return EditActionKind::ReplaceRange;
    }
    EditActionKind::ReplaceLastToken
}

#[cfg(test)]
mod tests {
    use super::{EditAction, EditActionKind};
    use crate::text_edit::{plan_committed_tail_full_token_replacement, plan_text_replacement};

    #[test]
    fn last_token_replacement_action_is_applyable() {
        let plan =
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan");
        let action = EditAction::planned_replacement(
            "typing-assist",
            700,
            "провека ",
            "проверка ",
            plan,
            Some("test"),
            Some("missing-letter"),
        );

        assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
        assert!(action.allow_apply());
        assert!(!action.boundary_changed());
    }

    #[test]
    fn unsafe_multiword_replacement_action_blocks_apply() {
        let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
        let action = EditAction::planned_replacement(
            "typing-assist",
            700,
            "одно два ",
            "однотри ",
            plan,
            Some("SemanticWordCell32"),
            Some("composite-typo"),
        );

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert!(action.boundary_changed());
    }

    #[test]
    fn ime_accept_candidate_is_a_planned_edit_action() {
        let action = EditAction::ime_accept(
            "ibus-active-composition-completion",
            900,
            "пров",
            "проверка ",
        );

        assert_eq!(action.kind, EditActionKind::AcceptImeCandidate);
        assert!(action.allow_apply());
        assert!(action.plan.is_some());
        assert!(action.safety.is_some());
        assert_eq!(
            action.transition.operator.as_deref(),
            Some("ime_active_composition_completion")
        );
        assert_eq!(
            action.transition.proof.as_deref(),
            Some("visible_preedit_completion_selected")
        );
        assert_eq!(action.transition.verified, Some(true));
        assert_eq!(
            action.selected_source_id.as_deref(),
            Some("ImeCompletionCell32")
        );
    }

    #[test]
    fn ime_accept_candidate_cannot_rewrite_left_context() {
        let action = EditAction::ime_accept(
            "ibus-active-composition-completion",
            900,
            "так можно проверить скры",
            "так можно проверять нкрытое сос",
        );

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "unsafe_multiword_autocorrect_scope");
        assert_eq!(
            action.transition.operator.as_deref(),
            Some("ime_active_composition_completion")
        );
    }

    #[test]
    fn low_confidence_boundary_split_blocks_apply() {
        let plan = plan_text_replacement("принамать ", "перинам ать ").expect("plan");
        let action = EditAction::planned_replacement(
            "typing-assist",
            225,
            "принамать ",
            "перинам ать ",
            plan,
            Some("BoundaryCell32"),
            Some("glued-words"),
        );

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "low_confidence_boundary_edit");
    }

    #[test]
    fn low_confidence_wide_boundary_edit_blocks_apply() {
        let plan = plan_text_replacement("сделай дома ", "с делай дома ").expect("plan");
        let action = EditAction::planned_replacement(
            "typing-assist",
            650,
            "сделай дома ",
            "с делай дома ",
            plan,
            Some("BoundaryCell32"),
            Some("glued-words"),
        );

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "low_confidence_wide_edit");
    }
}
