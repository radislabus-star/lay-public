use crate::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use crate::typing_candidate::rank_typing_candidates;
use crate::typing_rule_graph::ids;
use crate::word_reader::{split_edge_whitespace, split_ws_segments};

use super::candidates::evaluate_rule_candidates;
use super::types::TypingAssistExplanation;

pub(crate) fn collect_typing_assist_candidates_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Vec<(crate::typing_candidate::TypingCandidate, String)> {
    let (leading, core, trailing) = split_edge_whitespace(text);
    if core.is_empty() {
        return Vec::new();
    }
    evaluate_rule_candidates(
        TypingAssistExplanation::new(text, core, allow_layout_auto),
        core,
        allow_layout_auto,
        pipeline,
        true,
    )
    .candidates
    .into_iter()
    .filter(|candidate| !unsafe_word_count_shrink(core, &candidate.replacement, &candidate.rule_id))
    .filter_map(|candidate| {
        let mut output = String::with_capacity(text.len().max(candidate.replacement.len()));
        output.push_str(leading);
        output.push_str(&candidate.replacement);
        output.push_str(trailing);
        (output != text).then_some((candidate, output))
    })
    .collect()
}

pub fn select_typing_assist_exact(text: &str) -> Option<String> {
    select_typing_assist_with_pipeline(text, false, &default_typing_assist_pipeline())
}

pub fn select_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    let pipeline = default_typing_assist_pipeline();
    select_typing_assist_with_pipeline(text, allow_layout_auto, &pipeline)
}

pub fn select_typing_assist_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    let explanation = explain_typing_assist_with_pipeline(text, allow_layout_auto, pipeline);
    if std::env::var_os("LAY_TRACE_TYPING_ASSIST").is_some() {
        eprintln!("typing_assist_explanation={explanation:#?}");
    }
    explanation.output
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
        if unsafe_word_count_shrink(core, &decision.best.replacement, &decision.best.rule_id) {
            return explanation;
        }
        return explanation.with_decision(leading, trailing, decision);
    }

    let decision = rank_typing_candidates(candidates);
    let Some(decision) = decision else {
        return explanation;
    };
    if unsafe_word_count_shrink(core, &decision.best.replacement, &decision.best.rule_id) {
        return explanation;
    }
    explanation.with_decision(leading, trailing, decision)
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
