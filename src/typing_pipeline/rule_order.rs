use crate::config::{
    normalize_typing_assist_pipeline, sort_typing_assist_pipeline, TypingAssistRuleConfig,
};

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
    sort_typing_assist_pipeline(&mut rules);
    rules
}
