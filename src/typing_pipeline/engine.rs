use crate::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use crate::typing_candidate::{rank_typing_candidates, TypingCandidate, TypingCandidateDecision};
use crate::typing_rule_graph::{find_typing_rule, ids, priorities, rules, TypingRuleContext};
use crate::word_reader::{split_edge_whitespace, split_word_punctuation};

use super::rule_order::typing_rules_for_evaluation;
use super::types::{TypingAssistExplanation, TypingRuleEvaluation};

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
    let mut explanation = TypingAssistExplanation::new(text, core, allow_layout_auto);

    if core.is_empty() {
        return explanation;
    }

    let (token_leading, word, token_trailing) = split_word_punctuation(core);
    let ctx = TypingRuleContext {
        core,
        word,
        token_leading,
        token_trailing,
        allow_layout_auto,
    };

    if fast_en_to_ru_allowed(pipeline) {
        if let Some(replacement) = confident_en_to_ru_decision(&ctx) {
            let rule = TypingAssistRuleConfig {
                id: ids::FAST_LAYOUT_EN_TO_RU.to_string(),
                enabled: true,
                priority: priorities::FAST_LAYOUT_EN_TO_RU,
            };
            let candidate = TypingCandidate::new(
                ids::FAST_LAYOUT_EN_TO_RU,
                priorities::FAST_LAYOUT_EN_TO_RU,
                core,
                replacement,
            );
            explanation.record(TypingRuleEvaluation::new(&rule).with_candidate(candidate.clone()));
            return explanation.with_decision(
                leading,
                trailing,
                TypingCandidateDecision {
                    best: candidate,
                    second: None,
                    margin: f64::INFINITY,
                },
            );
        }
    }

    let mut candidates = Vec::new();

    for rule in typing_rules_for_evaluation(pipeline) {
        let evaluation = TypingRuleEvaluation::new(&rule);

        if !rule.enabled {
            explanation.record(evaluation.reject(TypingRuleEvaluation::REJECT_DISABLED));
            continue;
        }

        let Some(definition) = find_typing_rule(&rule.id) else {
            explanation.record(evaluation.reject(TypingRuleEvaluation::REJECT_UNKNOWN_RULE));
            continue;
        };

        let Some(replacement) = (definition.apply)(&ctx) else {
            explanation.record(evaluation.reject(TypingRuleEvaluation::REJECT_NO_CANDIDATE));
            continue;
        };

        let candidate = TypingCandidate::new(&rule.id, rule.priority, core, replacement);
        if !candidate.is_safe_for(core) {
            explanation.record(
                evaluation
                    .with_candidate(candidate)
                    .reject(TypingRuleEvaluation::REJECT_UNSAFE),
            );
            continue;
        }

        candidates.push(candidate.clone());
        explanation.record(evaluation.with_candidate(candidate));
    }

    let Some(decision) = rank_typing_candidates(candidates) else {
        return explanation;
    };
    explanation.with_decision(leading, trailing, decision)
}

fn confident_en_to_ru_decision(ctx: &TypingRuleContext<'_>) -> Option<String> {
    rules::apply_fast_layout_en_to_ru(ctx)
}

fn fast_en_to_ru_allowed(pipeline: &[TypingAssistRuleConfig]) -> bool {
    pipeline.iter().any(|rule| {
        rule.enabled
            && matches!(
                rule.id.as_str(),
                ids::CONTEXTUAL_LAYOUT_EN_TO_RU | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU
            )
    })
}
