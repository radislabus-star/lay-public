use crate::typing_candidate::TypingCandidateFamily;

use super::types::{
    TypingRuleContext, TypingRuleDefinition, TypingRuleRequiredSafety, TypingRuleSafety,
};
use super::weights::default_typing_rule_family_weight;

type TypingRuleApply = for<'a> fn(&TypingRuleContext<'a>) -> Option<String>;

pub(super) const fn rule(
    id: &'static str,
    default_priority: i32,
    family: TypingCandidateFamily,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(
        id,
        Some(default_priority),
        family,
        default_typing_rule_family_weight(family),
        TypingRuleSafety::Always,
        false,
        TypingRuleRequiredSafety::Strict,
        apply,
    )
}

pub(super) const fn weighted_rule(
    id: &'static str,
    default_priority: Option<i32>,
    family: TypingCandidateFamily,
    family_weight: f64,
    safety: TypingRuleSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(
        id,
        default_priority,
        family,
        family_weight,
        safety,
        false,
        TypingRuleRequiredSafety::Strict,
        apply,
    )
}

pub(super) const fn layout_only_rule(
    id: &'static str,
    default_priority: i32,
    safety: TypingRuleSafety,
    required_safety: TypingRuleRequiredSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(
        id,
        Some(default_priority),
        TypingCandidateFamily::Layout,
        default_typing_rule_family_weight(TypingCandidateFamily::Layout),
        safety,
        true,
        required_safety,
        apply,
    )
}

pub(super) const fn safety_rule(
    id: &'static str,
    default_priority: i32,
    family: TypingCandidateFamily,
    required_safety: TypingRuleRequiredSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    policy_rule(
        id,
        Some(default_priority),
        family,
        default_typing_rule_family_weight(family),
        TypingRuleSafety::Always,
        false,
        required_safety,
        apply,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) const fn policy_rule(
    id: &'static str,
    default_priority: Option<i32>,
    family: TypingCandidateFamily,
    family_weight: f64,
    safety: TypingRuleSafety,
    enabled_without_auto_replace: bool,
    required_safety: TypingRuleRequiredSafety,
    apply: TypingRuleApply,
) -> TypingRuleDefinition {
    TypingRuleDefinition {
        id,
        default_priority,
        family,
        family_weight,
        safety,
        enabled_without_auto_replace,
        required_safety,
        apply,
    }
}
