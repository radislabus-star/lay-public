use crate::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use crate::typing_candidate::rank_typing_candidates;
use crate::word_reader::split_edge_whitespace;

use super::candidates::evaluate_rule_candidates;
use super::types::TypingAssistExplanation;

pub fn apply_typing_assist_exact(text: &str) -> Option<String> {
    apply_typing_assist_with_pipeline(text, false, &default_typing_assist_pipeline())
}

pub fn apply_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    let pipeline = default_typing_assist_pipeline();
    apply_typing_assist_with_pipeline(text, allow_layout_auto, &pipeline)
}

pub fn apply_typing_assist_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    explain_typing_assist_with_pipeline(text, allow_layout_auto, pipeline).output
}

pub fn explain_typing_assist_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> TypingAssistExplanation {
    let (leading, core, trailing) = split_edge_whitespace(text);
    let explanation = TypingAssistExplanation::new(text, core, allow_layout_auto);

    if core.is_empty() {
        return explanation;
    }

    let collected = evaluate_rule_candidates(explanation, core, allow_layout_auto, pipeline, false);
    let explanation = collected.explanation;
    let candidates = collected.candidates;
    if let Some(decision) = collected.immediate_decision {
        return explanation.with_decision(leading, trailing, decision);
    }

    let decision = rank_typing_candidates(candidates);
    let Some(decision) = decision else {
        return explanation;
    };
    explanation.with_decision(leading, trailing, decision)
}
