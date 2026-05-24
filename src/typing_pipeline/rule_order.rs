use crate::config::{normalize_typing_assist_pipeline, TypingAssistRuleConfig};

pub(super) fn typing_rules_for_evaluation(
    pipeline: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
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
