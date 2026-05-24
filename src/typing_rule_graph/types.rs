use crate::typing_candidate::TypingCandidateFamily;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TypingRuleContext<'a> {
    pub core: &'a str,
    pub word: &'a str,
    pub token_leading: &'a str,
    pub token_trailing: &'a str,
    pub allow_layout_auto: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TypingRuleDefinition {
    pub id: &'static str,
    pub default_priority: Option<i32>,
    pub family: TypingCandidateFamily,
    pub family_weight: f64,
    pub safety: TypingRuleSafety,
    pub enabled_without_auto_replace: bool,
    pub required_safety: TypingRuleRequiredSafety,
    pub apply: for<'a> fn(&TypingRuleContext<'a>) -> Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypingRuleSafety {
    Always,
    PlainLayoutAutocorrect,
    ContextualLayoutAutocorrect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TypingRuleRequiredSafety {
    Strict,
    Normal,
    Experimental,
}
