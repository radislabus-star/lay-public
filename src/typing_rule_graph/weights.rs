use crate::typing_candidate::TypingCandidateFamily;

use super::registry::find_typing_rule;
use super::types::TypingRuleSafety;

pub(crate) fn typing_rule_family_weight(id: &str, family: TypingCandidateFamily) -> f64 {
    find_typing_rule(id)
        .map(|rule| rule.family_weight)
        .unwrap_or_else(|| default_typing_rule_family_weight(family))
}

pub(super) const fn default_typing_rule_family_weight(family: TypingCandidateFamily) -> f64 {
    match family {
        TypingCandidateFamily::Exact => 120.0,
        TypingCandidateFamily::Visual => 115.0,
        TypingCandidateFamily::Layout => 105.0,
        TypingCandidateFamily::Structural => 78.0,
        TypingCandidateFamily::Typo => 84.0,
        TypingCandidateFamily::Cleanup => 70.0,
        TypingCandidateFamily::Unknown => 40.0,
    }
}

pub(crate) fn typing_rule_candidate_is_safe(id: &str, original: &str, replacement: &str) -> bool {
    match find_typing_rule(id)
        .map(|rule| rule.safety)
        .unwrap_or(TypingRuleSafety::Always)
    {
        TypingRuleSafety::Always | TypingRuleSafety::ContextualLayoutAutocorrect => true,
        TypingRuleSafety::PlainLayoutAutocorrect => {
            !crate::word_recognizer::is_plain_layout_autocorrect_risky(original, replacement)
        }
    }
}
