//! After-space typing-assist pipeline.
//!
//! This module owns only rule ordering and candidate arbitration for completed
//! text passed in by the runtime. Smart manual scope correction can reuse it
//! without depending on the public `typing_assist` facade.

use crate::config::{
    default_typing_assist_pipeline, normalize_typing_assist_pipeline, TypingAssistRuleConfig,
};
use crate::typing_candidate::{choose_typing_candidate, TypingCandidate};
use crate::typing_rule_graph::{find_typing_rule, TypingRuleContext};
use crate::word_reader::{split_edge_whitespace, split_word_punctuation};

pub fn apply_typing_assist_exact(text: &str) -> Option<String> {
    apply_typing_assist_with_pipeline(text, false, &default_typing_assist_pipeline())
}

pub fn apply_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    let pipeline = default_typing_assist_pipeline();
    apply_typing_assist_with_pipeline(text, allow_layout_auto, &pipeline)
}

pub fn warm_up() {
    crate::lexicon::warm_up();
    crate::typing_replacements::warm_up();
    crate::layout_autoswitch::warm_up();
    crate::russian_lexicon::warm_up();
    crate::ngram::warm_up();
}

pub fn apply_typing_assist_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    explain_typing_assist_with_pipeline(text, allow_layout_auto, pipeline).output
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingAssistExplanation {
    pub original: String,
    pub core: String,
    pub allow_layout_auto: bool,
    pub evaluations: Vec<TypingRuleEvaluation>,
    pub chosen: Option<TypingCandidate>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingRuleEvaluation {
    pub id: String,
    pub priority: i32,
    pub enabled: bool,
    pub candidate: Option<TypingCandidate>,
    pub rejected: Option<String>,
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
        if !typing_assist_candidate_is_safe(&rule.id, core, &candidate.replacement) {
            evaluation.candidate = Some(candidate);
            evaluation.rejected = Some("unsafe autocorrect candidate".to_string());
            explanation.evaluations.push(evaluation);
            continue;
        }

        candidates.push(candidate.clone());
        evaluation.candidate = Some(candidate);
        explanation.evaluations.push(evaluation);
    }

    let Some(chosen) = choose_typing_candidate(candidates) else {
        return explanation;
    };

    let mut out = String::with_capacity(text.len().max(chosen.replacement.len()));
    out.push_str(leading);
    out.push_str(&chosen.replacement);
    out.push_str(trailing);
    if out != text {
        explanation.output = Some(out);
    }
    explanation.chosen = Some(chosen);
    explanation
}

fn typing_assist_candidate_is_safe(rule_id: &str, original: &str, replacement: &str) -> bool {
    if rule_id == "contextual_layout_en_to_ru" {
        return true;
    }
    if matches!(rule_id, "layout_ru_to_en" | "layout_en_to_ru") {
        return !crate::word_recognizer::is_plain_layout_autocorrect_risky(original, replacement);
    }
    true
}

fn typing_rules_for_evaluation(pipeline: &[TypingAssistRuleConfig]) -> Vec<TypingAssistRuleConfig> {
    let mut rules = normalize_typing_assist_pipeline(pipeline);
    for configured in pipeline {
        if rules.iter().any(|rule| rule.id == configured.id) {
            continue;
        }
        rules.push(configured.clone());
    }
    rules.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
    rules
}

#[cfg(test)]
#[path = "typing_pipeline_tests.rs"]
mod tests;
