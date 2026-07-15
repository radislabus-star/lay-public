use crate::text_edit::{
    committed_separator_is_preserved, ensure_committed_tail_spacing,
    offset_replacement_plan_for_cursor, plan_committed_tail_full_token_replacement,
    plan_committed_tail_last_token_replacement, plan_committed_tail_replacement,
    plan_decision_transition_edit, plan_text_replacement, replacement_plan_matches,
    DecisionTransitionEditInput, EditAction, TextReplacement, TransitionAudit,
};
use crate::typing_transition::decision::DecisionTransitionReceipt;

use super::types::{CorrectionSource, CorrectionTrigger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderEditPlan {
    pub trigger: CorrectionTrigger,
    pub original: String,
    pub replacement: String,
    pub plan: TextReplacement,
    pub source: CorrectionSource,
    pub(super) transition: TransitionAudit,
    selected_transition: Option<DecisionTransitionReceipt>,
    input_gate_trace: Option<crate::action_log::RecentActionGateTrace>,
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
            transition: TransitionAudit::none(),
            selected_transition: None,
            input_gate_trace: None,
            confidence_milli: 0,
            selected_source_id: None,
            selected_error_class: None,
        })
    }

    pub fn with_text_edit_input_gate_decision(
        mut self,
        decision: &crate::input_gate::InputGateDecision,
    ) -> Self {
        let selected_transition = decision
            .correction
            .as_ref()
            .and_then(|resolution| resolution.selected_transition.clone());
        let Some(selected_transition) = selected_transition else {
            return self;
        };
        let trace = decision.trace.as_ref();
        let confidence_milli = trace
            .and_then(|trace| trace.scoreboard.selected_bayes_posterior_milli)
            .unwrap_or(0);
        self.transition = selected_transition.diagnostic_transition();
        self.selected_transition = Some(selected_transition);
        self.confidence_milli = confidence_milli;
        self.selected_source_id = trace.and_then(|trace| trace.selected_source_id.clone());
        self.selected_error_class = trace
            .and_then(|trace| trace.selected_error_class)
            .map(|error_class| error_class.as_str().to_string());
        self.input_gate_trace =
            trace.map(crate::action_log::RecentActionGateTrace::from_input_gate);
        self
    }

    pub fn authorize_verified_replacement(
        &self,
        source: &str,
        original: &str,
        replacement: &str,
        plan: TextReplacement,
    ) -> EditAction {
        let Some(receipt) = self.selected_transition.as_ref() else {
            return EditAction::keep(source, original);
        };
        plan_decision_transition_edit(
            DecisionTransitionEditInput {
                source,
                confidence_milli: self.confidence_milli,
                from_text: original,
                to_text: replacement,
                plan,
                selected_source_id: self.selected_source_id.as_deref(),
                selected_error_class: self.selected_error_class.as_deref(),
            },
            receipt,
        )
    }

    pub fn plan_matches_replacement(&self) -> bool {
        replacement_plan_matches(&self.original, &self.replacement, &self.plan)
    }

    pub fn text_edit_input_gate_trace(&self) -> Option<&crate::action_log::RecentActionGateTrace> {
        self.input_gate_trace.as_ref()
    }

    pub fn preserves_committed_separator(&self) -> bool {
        if matches!(self.trigger, CorrectionTrigger::Manual) {
            return true;
        }
        committed_separator_is_preserved(&self.original, &self.replacement)
    }

    pub fn verified_plan_for_cursor(&self, cursor_offset: u32) -> Option<TextReplacement> {
        if self.selected_transition.is_none() || self.transition.blocks_apply() {
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
        if self.selected_transition.is_none() || self.transition.blocks_apply() {
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
