use crate::text_edit::{
    committed_separator_is_preserved, ensure_committed_tail_spacing,
    offset_replacement_plan_for_cursor, plan_committed_tail_full_token_replacement,
    plan_committed_tail_replacement, plan_text_replacement, replacement_plan_matches,
    TextReplacement,
};

use super::types::{CorrectionSource, CorrectionTrigger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderEditPlan {
    pub trigger: CorrectionTrigger,
    pub original: String,
    pub replacement: String,
    pub plan: TextReplacement,
    pub source: CorrectionSource,
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
            | CorrectionTrigger::Enter => plan_committed_tail_replacement(original, &replacement),
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
        })
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
