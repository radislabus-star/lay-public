use crate::config::TypingAssistRuleConfig;
use crate::typing_candidate::{TypingCandidate, TypingCandidateDecision};
use crate::typing_context::syntax_allows_candidate;
use crate::typing_replacements::replacement_for_token;
use crate::typing_rule_graph::{find_typing_rule, ids, priorities, rules, TypingRuleContext};
use crate::word_reader::{split_word_punctuation, split_ws_segments};
use std::time::Instant;

use super::rule_order::typing_rules_for_evaluation;
use super::types::{TypingAssistExplanation, TypingRuleEvaluation};

#[path = "candidates/safety.rs"]
mod safety;

const PERSONAL_REPLACEMENT_PRIORITY: i32 = 5;

pub(super) struct CandidateEvaluation {
    pub(super) explanation: TypingAssistExplanation,
    pub(super) candidates: Vec<TypingCandidate>,
    pub(super) immediate_decision: Option<TypingCandidateDecision>,
}

pub(super) fn evaluate_rule_candidates(
    mut explanation: TypingAssistExplanation,
    core: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    keep_fast_candidate_for_late_ranking: bool,
) -> CandidateEvaluation {
    let timing_enabled = std::env::var_os("LAY_DETERMINISTIC_CANDIDATE_TIMING").is_some();
    let total_started = Instant::now();
    let (token_leading, word, token_trailing) = split_word_punctuation(core);
    let ctx = TypingRuleContext {
        core,
        word,
        token_leading,
        token_trailing,
        allow_layout_auto,
    };
    let mut candidates = Vec::new();

    let promoted_started = Instant::now();
    if let Some((rule_id, replacement)) = promoted_replacement_candidate(&ctx) {
        let rule = TypingAssistRuleConfig {
            id: rule_id.to_string(),
            enabled: true,
            priority: PERSONAL_REPLACEMENT_PRIORITY,
        };
        let evaluation = TypingRuleEvaluation::new(&rule);
        let candidate =
            TypingCandidate::new(rule_id, PERSONAL_REPLACEMENT_PRIORITY, core, replacement);
        if !syntax_allows_candidate(core, &candidate.replacement)
            || safety::unsafe_word_count_shrink(core, &candidate.replacement, &candidate.rule_id)
        {
            explanation.record(
                evaluation
                    .with_candidate(candidate)
                    .reject(TypingRuleEvaluation::REJECT_UNSAFE),
            );
        } else if candidate.is_safe_for(core) {
            candidates.push(candidate.clone());
            explanation.record(evaluation.with_candidate(candidate));
        } else {
            explanation.record(
                evaluation
                    .with_candidate(candidate)
                    .reject(TypingRuleEvaluation::REJECT_UNSAFE),
            );
        }
    }
    let promoted_us = promoted_started.elapsed().as_micros();

    let fast_layout_started = Instant::now();
    if fast_en_to_ru_allowed(pipeline) {
        if let Some(replacement) = rules::apply_fast_layout_en_to_ru(&ctx) {
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
            if !keep_fast_candidate_for_late_ranking && candidates.is_empty() {
                return CandidateEvaluation {
                    explanation,
                    candidates,
                    immediate_decision: Some(TypingCandidateDecision {
                        best: candidate,
                        second: None,
                        margin: f64::INFINITY,
                    }),
                };
            }
            candidates.push(candidate);
        }
    }
    let fast_layout_us = fast_layout_started.elapsed().as_micros();

    let rules_started = Instant::now();
    let mut slowest_rule = String::new();
    let mut slowest_rule_us = 0_u128;
    let mut slow_rules = Vec::new();
    for rule in typing_rules_for_evaluation(pipeline) {
        let rule_started = Instant::now();
        let rule_id = rule.id.clone();
        (|| {
            let evaluation = TypingRuleEvaluation::new(&rule);
            if !rule.enabled {
                explanation.record(evaluation.reject(TypingRuleEvaluation::REJECT_DISABLED));
                return;
            }
            let Some(definition) = find_typing_rule(&rule.id) else {
                explanation.record(evaluation.reject(TypingRuleEvaluation::REJECT_UNKNOWN_RULE));
                return;
            };
            let Some(replacement) = (definition.apply)(&ctx) else {
                explanation.record(evaluation.reject(TypingRuleEvaluation::REJECT_NO_CANDIDATE));
                return;
            };
            let candidate = TypingCandidate::new(&rule.id, rule.priority, core, replacement);
            if !syntax_allows_candidate(core, &candidate.replacement)
                || safety::unsafe_word_count_shrink(
                    core,
                    &candidate.replacement,
                    &candidate.rule_id,
                )
            {
                explanation.record(
                    evaluation
                        .with_candidate(candidate)
                        .reject(TypingRuleEvaluation::REJECT_UNSAFE),
                );
            } else if candidate.is_safe_for(core) {
                candidates.push(candidate.clone());
                explanation.record(evaluation.with_candidate(candidate));
            } else {
                explanation.record(
                    evaluation
                        .with_candidate(candidate)
                        .reject(TypingRuleEvaluation::REJECT_UNSAFE),
                );
            }
        })();
        let rule_us = rule_started.elapsed().as_micros();
        if timing_enabled && rule_us >= 100 {
            slow_rules.push((rule_id.clone(), rule_us));
        }
        if rule_us > slowest_rule_us {
            slowest_rule = rule_id;
            slowest_rule_us = rule_us;
        }
    }
    let rules_us = rules_started.elapsed().as_micros();
    if timing_enabled {
        eprintln!(
            "lay_typing_rule_timing total_us={} promoted_us={} fast_layout_us={} rules_us={} slowest_rule={:?} slowest_rule_us={} slow_rules={:?} candidates={}",
            total_started.elapsed().as_micros(),
            promoted_us,
            fast_layout_us,
            rules_us,
            slowest_rule,
            slowest_rule_us,
            slow_rules,
            candidates.len(),
        );
    }

    CandidateEvaluation {
        explanation,
        candidates,
        immediate_decision: None,
    }
}

fn promoted_replacement_candidate(ctx: &TypingRuleContext<'_>) -> Option<(&'static str, String)> {
    if let Some(replacement) = replacement_for_token(ctx.core) {
        return Some((personal_rule_id(ctx.core, &replacement), replacement));
    }

    if !ctx.word.is_empty() && ctx.word != ctx.core {
        if let Some(replacement) = replacement_for_token(ctx.word) {
            let mut out = String::with_capacity(ctx.core.len().max(replacement.len()));
            out.push_str(ctx.token_leading);
            out.push_str(&replacement);
            out.push_str(ctx.token_trailing);
            return Some((personal_rule_id(ctx.core, &out), out));
        }
    }

    let segments = split_ws_segments(ctx.core);
    let (idx, leading, trailing, replacement) = segments
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, (_, is_ws))| !*is_ws)
        .find_map(|(idx, (segment, _))| {
            let (leading, word, trailing) = split_word_punctuation(segment);
            let replacement = replacement_for_token(word)?;
            Some((idx, leading, trailing, replacement))
        })?;
    let mut out = String::with_capacity(ctx.core.len().max(replacement.len()));
    for (segment_idx, (segment, _)) in segments.iter().enumerate() {
        if segment_idx == idx {
            out.push_str(leading);
            out.push_str(&replacement);
            out.push_str(trailing);
        } else {
            out.push_str(segment);
        }
    }
    Some((personal_rule_id(ctx.core, &out), out))
}

fn personal_rule_id(original: &str, replacement: &str) -> &'static str {
    if original.split_whitespace().count() > 1 || replacement.split_whitespace().count() > 1 {
        ids::PERSONAL_PHRASE
    } else {
        ids::PERSONAL_TOKEN
    }
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
