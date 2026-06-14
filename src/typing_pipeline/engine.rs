use crate::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use crate::microbrain::MicrobrainOptions;
use crate::typing_candidate::{rank_typing_candidates, rank_with_microbrain_trace};
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

pub fn apply_typing_assist_with_nanda(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    explain_typing_assist_with_nanda(text, allow_layout_auto, pipeline).output
}

pub fn explain_typing_assist_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> TypingAssistExplanation {
    explain_typing_assist_impl(text, allow_layout_auto, pipeline, None)
}

pub fn explain_typing_assist_with_nanda(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> TypingAssistExplanation {
    explain_typing_assist_impl(
        text,
        allow_layout_auto,
        pipeline,
        Some(&MicrobrainOptions::default()),
    )
}

pub fn explain_typing_assist_with_microbrain_options(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    options: &MicrobrainOptions,
) -> TypingAssistExplanation {
    explain_typing_assist_impl(text, allow_layout_auto, pipeline, Some(options))
}

fn explain_typing_assist_impl(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    microbrain_options: Option<&MicrobrainOptions>,
) -> TypingAssistExplanation {
    let (leading, core, trailing) = split_edge_whitespace(text);
    let explanation = TypingAssistExplanation::new(text, core, allow_layout_auto);

    if core.is_empty() {
        return explanation;
    }

    let collected = evaluate_rule_candidates(
        explanation,
        core,
        allow_layout_auto,
        pipeline,
        microbrain_options.is_some(),
    );
    let explanation = collected.explanation;
    let candidates = collected.candidates;
    if let Some(decision) = collected.immediate_decision {
        return explanation.with_decision(leading, trailing, decision);
    }

    if let Some(options) = microbrain_options {
        let (decision, trace) = rank_with_microbrain_trace(core, candidates, options);
        let explanation = if let Some(decision) = decision {
            explanation.with_decision(leading, trailing, decision)
        } else {
            explanation
        };
        return explanation.with_microbrain(trace);
    }

    let decision = rank_typing_candidates(candidates);
    let Some(decision) = decision else {
        return explanation;
    };
    explanation.with_decision(leading, trailing, decision)
}
