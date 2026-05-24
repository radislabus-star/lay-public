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
    let mut explanation = TypingAssistExplanation {
        original: text.to_string(),
        core: core.to_string(),
        allow_layout_auto,
        evaluations: Vec::new(),
        chosen: None,
        second: None,
        margin: None,
        output: None,
    };

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
        let mut evaluation = TypingRuleEvaluation {
            id: rule.id.clone(),
            priority: rule.priority,
            enabled: rule.enabled,
            candidate: None,
            rejected: None,
        };

        if !rule.enabled {
            evaluation.rejected = Some("disabled".to_string());
            explanation.evaluations.push(evaluation);
            continue;
        }

        let Some(definition) = find_typing_rule(&rule.id) else {
            evaluation.rejected = Some("unknown rule".to_string());
            explanation.evaluations.push(evaluation);
            continue;
        };

        let Some(replacement) = (definition.apply)(&ctx) else {
            evaluation.rejected = Some("no candidate".to_string());
            explanation.evaluations.push(evaluation);
            continue;
        };

        let candidate = TypingCandidate::new(&rule.id, rule.priority, core, replacement);
        if !crate::typing_rule_graph::typing_rule_candidate_is_safe(
            &rule.id,
            core,
            &candidate.replacement,
        ) {
            evaluation.candidate = Some(candidate);
            evaluation.rejected = Some("unsafe autocorrect candidate".to_string());
            explanation.evaluations.push(evaluation);
            continue;
        }

        candidates.push(candidate.clone());
        evaluation.candidate = Some(candidate);
        explanation.evaluations.push(evaluation);
    }

    let Some(decision) = rank_typing_candidates(candidates) else {
        return explanation;
    };
    let chosen = decision.best;

    let mut out = String::with_capacity(text.len().max(chosen.replacement.len()));
    out.push_str(leading);
    out.push_str(&chosen.replacement);
    out.push_str(trailing);
    if out != text {
        explanation.output = Some(out);
    }
    explanation.second = decision.second;
    explanation.margin = Some(decision.margin);
    explanation.chosen = Some(chosen);
    explanation
}
