use super::gate::VerifiedTransitionReceipt;
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
    kind: EditActionKind,
    source: String,
    confidence_milli: i16,
    from_text: String,
    to_text: String,
    plan: Option<TextReplacement>,
    safety: Option<EditPlanSafetyReport>,
    transition: TransitionAudit,
    verification: Option<VerifiedTransitionReceipt>,
    selected_source_id: Option<String>,
    selected_error_class: Option<String>,
}

pub(crate) struct PlannedReplacementInput<'a> {
    pub(crate) source: &'a str,
    pub(crate) confidence_milli: i16,
    pub(crate) from_text: &'a str,
    pub(crate) to_text: &'a str,
    pub(crate) plan: TextReplacement,
    pub(crate) selected_source_id: Option<&'a str>,
    pub(crate) selected_error_class: Option<&'a str>,
    pub(crate) transition: TransitionAudit,
}

pub(crate) struct DecisionTransitionEditInput<'a> {
    pub(crate) source: &'a str,
    pub(crate) confidence_milli: i16,
    pub(crate) from_text: &'a str,
    pub(crate) to_text: &'a str,
    pub(crate) plan: TextReplacement,
    pub(crate) selected_source_id: Option<&'a str>,
    pub(crate) selected_error_class: Option<&'a str>,
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
            verification: None,
            selected_source_id: None,
            selected_error_class: None,
        }
    }

    pub(super) fn planned_replacement(input: PlannedReplacementInput<'_>) -> Self {
        let PlannedReplacementInput {
            source,
            confidence_milli,
            from_text,
            to_text,
            plan,
            selected_source_id,
            selected_error_class,
            transition,
        } = input;
        let from_text = from_text.to_string();
        let to_text = to_text.to_string();
        let mut safety = autocorrect_edit_safety(&from_text, &to_text, &plan, &transition);
        downgrade_low_confidence_boundary_edit(&mut safety, confidence_milli);
        downgrade_low_confidence_wide_edit(&mut safety, confidence_milli, selected_error_class);
        let kind = classify_planned_replacement(&safety);
        Self {
            kind,
            source: source.to_string(),
            confidence_milli,
            from_text,
            to_text,
            plan: Some(plan),
            safety: Some(safety),
            transition,
            verification: None,
            selected_source_id: selected_source_id.map(str::to_string),
            selected_error_class: selected_error_class.map(str::to_string),
        }
    }

    pub(super) fn attach_verification(&mut self, receipt: VerifiedTransitionReceipt) {
        self.verification = Some(receipt);
    }

    pub(super) fn mark_ime_accept(&mut self) {
        self.kind = EditActionKind::AcceptImeCandidate;
    }

    pub fn allow_apply(&self) -> bool {
        self.safety_allows_apply()
            && self
                .verification
                .as_ref()
                .is_some_and(|receipt| receipt.matches(self))
    }

    pub const fn kind(&self) -> EditActionKind {
        self.kind
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub const fn confidence_milli(&self) -> i16 {
        self.confidence_milli
    }

    pub fn from_text(&self) -> &str {
        &self.from_text
    }

    pub fn to_text(&self) -> &str {
        &self.to_text
    }

    pub fn plan(&self) -> Option<&TextReplacement> {
        self.plan.as_ref()
    }

    pub fn safety(&self) -> Option<&EditPlanSafetyReport> {
        self.safety.as_ref()
    }

    pub fn transition(&self) -> &TransitionAudit {
        &self.transition
    }

    pub fn selected_source_id(&self) -> Option<&str> {
        self.selected_source_id.as_deref()
    }

    pub fn selected_error_class(&self) -> Option<&str> {
        self.selected_error_class.as_deref()
    }

    pub(crate) fn has_verifier_receipt(&self) -> bool {
        self.verification
            .as_ref()
            .is_some_and(|receipt| receipt.matches(self))
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
        if !self.safety_allows_apply() {
            return Some(self.safety_reason());
        }
        if self.plan.is_none() {
            return Some("verified_edit_plan_missing");
        }
        if !self.has_verifier_receipt() {
            return Some("verified_transition_receipt_missing");
        }
        None
    }

    fn safety_allows_apply(&self) -> bool {
        self.safety
            .as_ref()
            .is_some_and(|safety| safety.allow_apply)
            && !matches!(self.kind, EditActionKind::BlockUnsafe)
            && self.plan.is_some()
            && self.transition.is_verified()
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
    use super::{EditAction, EditActionKind, PlannedReplacementInput};
    use crate::text_edit::{
        plan_committed_tail_full_token_replacement, plan_ime_candidate_accept_edit,
        plan_text_replacement, TransitionAudit, TransitionOperator, TransitionProof,
    };

    #[test]
    fn unsealed_last_token_replacement_has_no_apply_authority() {
        let plan =
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan");
        let action = EditAction::planned_replacement(PlannedReplacementInput {
            source: "typing-assist",
            confidence_milli: 700,
            from_text: "провека ",
            to_text: "проверка ",
            plan,
            selected_source_id: Some("test"),
            selected_error_class: Some("missing-letter"),
            transition: TransitionAudit::proven(
                TransitionOperator::ReplaceCurrentWord,
                TransitionProof::Typo,
                true,
                false,
                1,
            ),
        });

        assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
        assert!(!action.allow_apply());
        assert!(!action.has_verifier_receipt());
        assert!(!action.boundary_changed());
    }

    #[test]
    fn unsafe_multiword_replacement_action_blocks_apply() {
        let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
        let action = EditAction::planned_replacement(PlannedReplacementInput {
            source: "typing-assist",
            confidence_milli: 700,
            from_text: "одно два ",
            to_text: "однотри ",
            plan,
            selected_source_id: Some("SemanticWordCell32"),
            selected_error_class: Some("composite-typo"),
            transition: TransitionAudit::proven(
                TransitionOperator::PhraseTokenRepair,
                TransitionProof::Context,
                true,
                true,
                2,
            ),
        });

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert!(action.boundary_changed());
    }

    #[test]
    fn ime_accept_candidate_is_a_planned_edit_action() {
        let action = plan_ime_candidate_accept_edit(
            "ibus-active-composition-candidate-accept",
            900,
            "пров",
            "проверка ",
        );

        assert_eq!(action.kind, EditActionKind::AcceptImeCandidate);
        assert!(action.allow_apply());
        assert!(action.plan.is_some());
        assert!(action.safety.is_some());
        assert_eq!(
            action.transition.operator(),
            Some(TransitionOperator::Completion)
        );
        assert_eq!(action.transition.proof(), Some(TransitionProof::Completion));
        assert_eq!(action.transition.verified(), Some(true));
        assert_eq!(
            action.selected_source_id.as_deref(),
            Some("ImeCandidateAcceptCell32")
        );
    }

    #[test]
    fn ime_accept_candidate_allows_an_explicit_full_token_replacement() {
        let action = plan_ime_candidate_accept_edit(
            "ibus-active-composition-candidate-accept",
            900,
            "провв",
            "проверка ",
        );

        assert_eq!(action.kind, EditActionKind::AcceptImeCandidate);
        assert!(action.allow_apply());
        assert_eq!(
            action.transition.operator(),
            Some(TransitionOperator::ManualReplace)
        );
        assert_eq!(
            action.transition.proof(),
            Some(TransitionProof::ManualIntent)
        );
        assert_eq!(
            action.plan,
            Some(crate::text_edit::TextReplacement {
                move_left: 0,
                backspaces: 5,
                insert: "проверка ".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn ime_accept_candidate_allows_surface_preserving_boundary_split() {
        let action = plan_ime_candidate_accept_edit(
            "ibus-active-composition-candidate-accept",
            900,
            "Еленапросит",
            "Елена просит ",
        );

        assert_eq!(action.kind, EditActionKind::AcceptImeCandidate);
        assert!(action.allow_apply());
        assert!(action.boundary_changed());
    }

    #[test]
    fn ime_accept_candidate_cannot_rewrite_left_context() {
        let action = plan_ime_candidate_accept_edit(
            "ibus-active-composition-candidate-accept",
            900,
            "так можно проверить скры",
            "так можно проверять нкрытое сос",
        );

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "unsafe_multiword_autocorrect_scope");
        assert_eq!(action.transition.operator(), None);
    }

    #[test]
    fn low_confidence_boundary_split_blocks_apply() {
        let plan = plan_text_replacement("принамать ", "перинам ать ").expect("plan");
        let action = EditAction::planned_replacement(PlannedReplacementInput {
            source: "typing-assist",
            confidence_milli: 225,
            from_text: "принамать ",
            to_text: "перинам ать ",
            plan,
            selected_source_id: Some("BoundaryCell32"),
            selected_error_class: Some("glued-words"),
            transition: TransitionAudit::proven(
                TransitionOperator::BoundaryMergeSplit,
                TransitionProof::Boundary,
                true,
                true,
                2,
            ),
        });

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "low_confidence_boundary_edit");
    }

    #[test]
    fn low_confidence_wide_boundary_edit_blocks_apply() {
        let plan = plan_text_replacement("сделай дома ", "с делай дома ").expect("plan");
        let action = EditAction::planned_replacement(PlannedReplacementInput {
            source: "typing-assist",
            confidence_milli: 650,
            from_text: "сделай дома ",
            to_text: "с делай дома ",
            plan,
            selected_source_id: Some("BoundaryCell32"),
            selected_error_class: Some("glued-words"),
            transition: TransitionAudit::proven(
                TransitionOperator::BoundaryMergeSplit,
                TransitionProof::Boundary,
                true,
                true,
                2,
            ),
        });

        assert_eq!(action.kind, EditActionKind::BlockUnsafe);
        assert!(!action.allow_apply());
        assert_eq!(action.safety_reason(), "low_confidence_wide_edit");
    }
}
