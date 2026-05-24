use super::{default_typing_assist_pipeline, CorrectionSafety, TypingAssistRuleConfig};

pub fn normalize_typing_assist_pipeline(
    configured: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
    let mut rules = default_typing_assist_pipeline();
    for saved in configured {
        if let Some(rule) = rules.iter_mut().find(|rule| rule.id == saved.id) {
            rule.enabled = saved.enabled;
            if saved.priority > 0 {
                rule.priority = saved.priority;
            }
        }
    }
    rules.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
    rules
}

pub fn typing_assist_pipeline_for_auto_replace(
    auto_replace: bool,
    configured: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
    typing_assist_pipeline_for_policy(auto_replace, CorrectionSafety::Normal, configured)
}

pub fn typing_assist_pipeline_for_policy(
    auto_replace: bool,
    safety: CorrectionSafety,
    configured: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
    let mut rules = normalize_typing_assist_pipeline(configured);
    apply_auto_replace_policy(&mut rules, auto_replace, safety);
    rules
}

fn apply_auto_replace_policy(
    rules: &mut [TypingAssistRuleConfig],
    auto_replace: bool,
    safety: CorrectionSafety,
) {
    if !auto_replace {
        for rule in rules {
            rule.enabled = rule.enabled
                && crate::typing_rule_graph::typing_rule_enabled_without_auto_replace(&rule.id);
        }
        return;
    }

    for rule in rules {
        rule.enabled = rule.enabled && rule_allowed_by_safety(&rule.id, safety);
    }
}

fn rule_allowed_by_safety(id: &str, safety: CorrectionSafety) -> bool {
    use crate::typing_rule_graph::TypingRuleRequiredSafety;

    match crate::typing_rule_graph::typing_rule_required_safety(id) {
        TypingRuleRequiredSafety::Strict => true,
        TypingRuleRequiredSafety::Normal => safety != CorrectionSafety::Strict,
        TypingRuleRequiredSafety::Experimental => safety == CorrectionSafety::Experimental,
    }
}
