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
        let kind = classify_planned_replacement(&safety);
        Self {
            kind,
            source: source.into(),
            confidence_milli,
            from_text,
            to_text,
            plan: Some(plan),
            safety: Some(safety),
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
        Self {
            kind: EditActionKind::AcceptImeCandidate,
            source: source.into(),
            confidence_milli,
            from_text: from_text.into(),
            to_text: to_text.into(),
            plan: None,
            safety: None,
            selected_source_id: None,
            selected_error_class: None,
        }
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
}
