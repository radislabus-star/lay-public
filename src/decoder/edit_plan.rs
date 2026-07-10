use crate::text_edit::{
    authorize_replacement_with_transition, committed_separator_is_preserved,
    ensure_committed_tail_spacing, offset_replacement_plan_for_cursor,
    plan_committed_tail_full_token_replacement, plan_committed_tail_last_token_replacement,
    plan_committed_tail_replacement, plan_text_replacement, replacement_plan_matches, EditAction,
    TextReplacement, TransitionAudit,
};

use super::types::{CorrectionSource, CorrectionTrigger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderEditPlan {
    pub trigger: CorrectionTrigger,
    pub original: String,
    pub replacement: String,
    pub plan: TextReplacement,
    pub source: CorrectionSource,
    transition: TransitionAudit,
    confidence_milli: i16,
    selected_source_id: Option<String>,
    selected_error_class: Option<String>,
}

impl DecoderEditPlan {
    pub fn committed_tail(
        trigger: CorrectionTrigger,
        original: &str,
        replacement: &str,
        source: CorrectionSource,
    ) -> Option<Self> {
        let replacement = ensure_committed_tail_spacing(original, replacement.to_string());
        let plan = match trigger {
            CorrectionTrigger::AfterSpace
            | CorrectionTrigger::AfterPunctuation
            | CorrectionTrigger::Enter => {
                plan_committed_tail_last_token_replacement(original, &replacement)
                    .or_else(|| plan_committed_tail_replacement(original, &replacement))
            }
            CorrectionTrigger::Manual => plan_text_replacement(original, &replacement),
        }?;
        debug_assert!(
            replacement_plan_matches(original, &replacement, &plan),
            "decoder edit plan must apply exactly to replacement"
        );
        debug_assert!(
            !matches!(
                trigger,
                CorrectionTrigger::AfterSpace
                    | CorrectionTrigger::AfterPunctuation
                    | CorrectionTrigger::Enter
            ) || committed_separator_is_preserved(original, &replacement),
            "committed correction must preserve typed separator"
        );

        Some(Self {
            trigger,
            original: original.to_string(),
            replacement,
            plan,
            source,
            transition: TransitionAudit::proven(
                "decoder_committed_tail",
                "decoder_tail_plan_verified",
                true,
                false,
                original.split_whitespace().count().max(1),
            ),
            confidence_milli: 0,
            selected_source_id: None,
            selected_error_class: None,
        })
    }

    fn with_transition_audit(
        mut self,
        transition: TransitionAudit,
        confidence_milli: i16,
        selected_source_id: Option<&str>,
        selected_error_class: Option<&str>,
    ) -> Self {
        self.transition = transition;
        self.confidence_milli = confidence_milli;
        self.selected_source_id = selected_source_id.map(str::to_string);
        self.selected_error_class = selected_error_class.map(str::to_string);
        self
    }

    pub fn with_input_gate_trace(self, trace: &crate::action_log::RecentActionGateTrace) -> Self {
        let confidence_milli = trace
            .scoreboard
            .as_ref()
            .and_then(|scoreboard| scoreboard.selected_bayes_posterior_milli)
            .unwrap_or(0);
        self.with_transition_audit(
            trace.selected_transition_audit(),
            confidence_milli,
            trace.selected_source_id.as_deref(),
            trace.selected_error_class.as_deref(),
        )
    }

    pub fn authorize_verified_replacement(
        &self,
        source: &str,
        original: &str,
        replacement: &str,
        plan: TextReplacement,
    ) -> EditAction {
        authorize_replacement_with_transition(
            source,
            self.confidence_milli,
            original,
            replacement,
            plan,
            self.selected_source_id.as_deref(),
            self.selected_error_class.as_deref(),
            self.transition.clone(),
        )
    }

    pub fn plan_matches_replacement(&self) -> bool {
        replacement_plan_matches(&self.original, &self.replacement, &self.plan)
    }

    pub fn preserves_committed_separator(&self) -> bool {
        if matches!(self.trigger, CorrectionTrigger::Manual) {
            return true;
        }
        committed_separator_is_preserved(&self.original, &self.replacement)
    }

    pub fn verified_plan_for_cursor(&self, cursor_offset: u32) -> Option<TextReplacement> {
        if self.transition.blocks_apply() {
            return None;
        }
        if !self.plan_matches_replacement() || !self.preserves_committed_separator() {
            return None;
        }
        Some(offset_replacement_plan_for_cursor(
            &self.plan,
            cursor_offset,
        ))
    }

    pub fn verified_full_token_plan_for_cursor(
        &self,
        cursor_offset: u32,
    ) -> Option<TextReplacement> {
        if self.transition.blocks_apply() {
            return None;
        }
        if !self.plan_matches_replacement() || !self.preserves_committed_separator() {
            return None;
        }
        let plan = plan_committed_tail_full_token_replacement(&self.original, &self.replacement)?;
        if !replacement_plan_matches(&self.original, &self.replacement, &plan) {
            return None;
        }
        Some(offset_replacement_plan_for_cursor(&plan, cursor_offset))
    }
}
