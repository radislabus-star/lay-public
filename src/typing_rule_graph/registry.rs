use crate::typing_candidate::TypingCandidateFamily;

use super::definitions::RULES;
use super::types::{TypingRuleDefinition, TypingRuleRequiredSafety};

pub(crate) fn typing_rule_definitions() -> &'static [TypingRuleDefinition] {
    RULES
}

pub(crate) fn find_typing_rule(id: &str) -> Option<&'static TypingRuleDefinition> {
    typing_rule_definitions().iter().find(|rule| rule.id == id)
}

pub(crate) fn typing_rule_family(id: &str) -> Option<TypingCandidateFamily> {
    find_typing_rule(id).map(|rule| rule.family)
}

pub(crate) fn typing_rule_enabled_without_auto_replace(id: &str) -> bool {
    find_typing_rule(id).is_some_and(|rule| rule.enabled_without_auto_replace)
}

pub(crate) fn typing_rule_required_safety(id: &str) -> TypingRuleRequiredSafety {
    find_typing_rule(id)
        .map(|rule| rule.required_safety)
        .unwrap_or(TypingRuleRequiredSafety::Experimental)
}
