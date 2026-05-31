use crate::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use crate::typing_candidate::{rank_typing_candidates, TypingCandidate};
use crate::typing_rule_graph::{find_typing_rule, TypingRuleContext};
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
