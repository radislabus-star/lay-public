use crate::config::TypingAssistRuleConfig;
use crate::typing_candidate::{TypingCandidate, TypingCandidateDecision};
use crate::typing_context::syntax_allows_candidate;
use crate::typing_rule_graph::{find_typing_rule, ids, priorities, rules, TypingRuleContext};
use crate::word_reader::{split_word_punctuation, split_ws_segments};

use super::rule_order::typing_rules_for_evaluation;
use super::types::{TypingAssistExplanation, TypingRuleEvaluation};

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
    let (token_leading, word, token_trailing) = split_word_punctuation(core);
    let ctx = TypingRuleContext {
        core,
        word,
        token_leading,
        token_trailing,
        allow_layout_auto,
    };
    let mut candidates = Vec::new();

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
            if !keep_fast_candidate_for_late_ranking {
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
        if !syntax_allows_candidate(core, &candidate.replacement)
            || unsafe_word_count_shrink(core, &candidate.replacement, &candidate.rule_id)
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

    CandidateEvaluation {
        explanation,
        candidates,
        immediate_decision: None,
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

fn unsafe_word_count_shrink(original: &str, replacement: &str, rule_id: &str) -> bool {
    if matches!(
        rule_id,
        ids::SPLIT_WORD_PAIR | ids::GLUED_PHRASE | ids::MOVED_PREFIX_PAIR
    ) {
        return false;
    }
    let original_words = split_ws_segments(original)
        .into_iter()
        .filter(|(_, is_ws)| !*is_ws)
        .count();
    let replacement_words = split_ws_segments(replacement)
        .into_iter()
        .filter(|(_, is_ws)| !*is_ws)
        .count();
    original_words >= 2 && replacement_words < original_words
}
