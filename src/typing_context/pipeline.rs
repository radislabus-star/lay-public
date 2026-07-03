use crate::config::{
    normalize_typing_assist_pipeline, sort_typing_assist_pipeline,
    typing_assist_pipeline_for_policy, CorrectionSafety, TypingAssistRuleConfig,
};
use crate::typing_rule_graph::ids::{
    CONTEXTUAL_LAYOUT_EN_TO_RU, EXPERIMENTAL_LAYOUT_EN_TO_RU, EXPERIMENTAL_LAYOUT_RU_TO_EN,
    LAYOUT_EN_TO_RU, LAYOUT_RU_TO_EN,
};

use super::layout_signal::should_enable_ascii_to_ru_layout;

pub fn typing_assist_pipeline_for_context(
    auto_replace: bool,
    safety: CorrectionSafety,
    configured: &[TypingAssistRuleConfig],
    context: &str,
) -> Vec<TypingAssistRuleConfig> {
    let mut pipeline = typing_assist_pipeline_for_policy(auto_replace, safety, configured);
    if auto_replace
        && safety == CorrectionSafety::Normal
        && should_enable_ascii_to_ru_layout(context)
        && user_config_allows_rule(configured, LAYOUT_EN_TO_RU)
    {
        push_contextual_rule(&mut pipeline, CONTEXTUAL_LAYOUT_EN_TO_RU);
    }
    if auto_replace
        && safety == CorrectionSafety::Experimental
        && user_config_allows_rule(configured, LAYOUT_EN_TO_RU)
    {
        push_contextual_rule(&mut pipeline, EXPERIMENTAL_LAYOUT_EN_TO_RU);
    }
    if auto_replace
        && safety == CorrectionSafety::Experimental
        && user_config_allows_rule(configured, LAYOUT_RU_TO_EN)
    {
        push_contextual_rule(&mut pipeline, EXPERIMENTAL_LAYOUT_RU_TO_EN);
    }
    pipeline
}

fn push_contextual_rule(pipeline: &mut Vec<TypingAssistRuleConfig>, id: &str) {
    let priority = if id == EXPERIMENTAL_LAYOUT_RU_TO_EN {
        contextual_priority(pipeline, LAYOUT_RU_TO_EN, 89)
    } else {
        contextual_priority(pipeline, LAYOUT_EN_TO_RU, 99)
    };
    pipeline.push(TypingAssistRuleConfig {
        id: id.to_string(),
        enabled: true,
        priority,
    });
    sort_typing_assist_pipeline(pipeline);
}

fn user_config_allows_rule(configured: &[TypingAssistRuleConfig], id: &str) -> bool {
    if configured.iter().any(|rule| rule.id == id && rule.enabled) {
        return true;
    }
    if id == LAYOUT_EN_TO_RU
        && configured.iter().any(|rule| {
            matches!(
                rule.id.as_str(),
                CONTEXTUAL_LAYOUT_EN_TO_RU | EXPERIMENTAL_LAYOUT_EN_TO_RU
            ) && rule.enabled
        })
    {
        return true;
    }
    if id == LAYOUT_RU_TO_EN
        && configured
            .iter()
            .any(|rule| rule.id == EXPERIMENTAL_LAYOUT_RU_TO_EN && rule.enabled)
    {
        return true;
    }
    normalize_typing_assist_pipeline(configured)
        .iter()
        .find(|rule| rule.id == id)
        .is_some_and(|rule| rule.enabled)
}

fn contextual_priority(pipeline: &[TypingAssistRuleConfig], id: &str, fallback: i32) -> i32 {
    pipeline
        .iter()
        .find(|rule| rule.id == id)
        .map(|rule| rule.priority.saturating_sub(1).max(1))
        .unwrap_or(fallback)
}
