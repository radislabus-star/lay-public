use crate::typing_candidate::TypingCandidateFamily;

use super::types::{
    TypingRuleContext, TypingRuleDefinition, TypingRuleRequiredSafety, TypingRuleSafety,
};
use super::weights::default_typing_rule_family_weight;

type TypingRuleApply = for<'a> fn(&TypingRuleContext<'a>) -> Option<String>;

pub(super) struct RulePolicy {
    pub id: &'static str,
    pub default_priority: Option<i32>,
    pub family: TypingCandidateFamily,
    pub family_weight: f64,
    pub safety: TypingRuleSafety,
    pub enabled_without_auto_replace: bool,
    pub required_safety: TypingRuleRequiredSafety,
    pub apply: TypingRuleApply,
}

pub(super) const fn rule(
    id: &'static str,
    default_priority: i32,
    family: TypingCandidateFamily,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(RulePolicy {
        id,
        default_priority: Some(default_priority),
        family,
        family_weight: default_typing_rule_family_weight(family),
        safety: TypingRuleSafety::Always,
        enabled_without_auto_replace: false,
        required_safety: TypingRuleRequiredSafety::Strict,
        apply,
    })
}

pub(super) const fn weighted_rule(
    id: &'static str,
    default_priority: Option<i32>,
    family: TypingCandidateFamily,
    family_weight: f64,
    safety: TypingRuleSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(RulePolicy {
        id,
        default_priority,
        family,
        family_weight,
        safety,
        enabled_without_auto_replace: false,
        required_safety: TypingRuleRequiredSafety::Strict,
        apply,
    })
}

pub(super) const fn layout_only_rule(
    id: &'static str,
    default_priority: i32,
    safety: TypingRuleSafety,
    required_safety: TypingRuleRequiredSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(RulePolicy {
        id,
        default_priority: Some(default_priority),
        family: TypingCandidateFamily::Layout,
        family_weight: default_typing_rule_family_weight(TypingCandidateFamily::Layout),
        safety,
        enabled_without_auto_replace: true,
        required_safety,
        apply,
    })
}

pub(super) const fn contextual_layout_rule(
    id: &'static str,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(RulePolicy {
        id,
        default_priority: None,
        family: TypingCandidateFamily::Layout,
        family_weight: default_typing_rule_family_weight(TypingCandidateFamily::Layout),
        safety: TypingRuleSafety::ContextualLayoutAutocorrect,
        enabled_without_auto_replace: false,
        required_safety: TypingRuleRequiredSafety::Strict,
        apply,
    })
}

pub(super) const fn experimental_weighted_rule(
    id: &'static str,
    default_priority: i32,
    family: TypingCandidateFamily,
    family_weight: f64,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(RulePolicy {
        id,
        default_priority: Some(default_priority),
        family,
        family_weight,
        safety: TypingRuleSafety::Always,
        enabled_without_auto_replace: false,
        required_safety: TypingRuleRequiredSafety::Experimental,
        apply,
    })
}

pub(super) const fn safety_rule(
    id: &'static str,
    default_priority: i32,
    family: TypingCandidateFamily,
    required_safety: TypingRuleRequiredSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(RulePolicy {
        id,
        default_priority: Some(default_priority),
        family,
        family_weight: default_typing_rule_family_weight(family),
        safety: TypingRuleSafety::Always,
        enabled_without_auto_replace: false,
        required_safety,
        apply,
    })
}

pub(super) const fn policy_rule(policy: RulePolicy) -> TypingRuleDefinition {
    TypingRuleDefinition {
        id: policy.id,
        default_priority: policy.default_priority,
        family: policy.family,
        family_weight: policy.family_weight,
        safety: policy.safety,
        enabled_without_auto_replace: policy.enabled_without_auto_replace,
        required_safety: policy.required_safety,
        apply: policy.apply,
    }
}
