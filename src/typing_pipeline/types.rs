use crate::config::TypingAssistRuleConfig;
use crate::typing_candidate::{
    classify_typing_confidence, TypingCandidate, TypingCandidateDecision, TypingDecisionConfidence,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypingAssistExplanation {
    pub original: String,
    pub core: String,
    pub allow_layout_auto: bool,
    pub evaluations: Vec<TypingRuleEvaluation>,
    pub chosen: Option<TypingCandidate>,
    pub second: Option<TypingCandidate>,
    pub margin: Option<f64>,
    pub output: Option<String>,
}

impl TypingAssistExplanation {
    pub(crate) fn new(original: &str, core: &str, allow_layout_auto: bool) -> Self {
        Self {
            original: original.to_string(),
            core: core.to_string(),
            allow_layout_auto,
            evaluations: Vec::new(),
            chosen: None,
            second: None,
            margin: None,
            output: None,
        }
    }

    pub(crate) fn record(&mut self, evaluation: TypingRuleEvaluation) {
        self.evaluations.push(evaluation);
    }

    pub(crate) fn with_decision(
        mut self,
        leading: &str,
        trailing: &str,
        decision: TypingCandidateDecision,
    ) -> Self {
        let chosen = decision.best;
        let mut out = String::with_capacity(self.original.len().max(chosen.replacement.len()));
        out.push_str(leading);
        out.push_str(&chosen.replacement);
        out.push_str(trailing);
        if out != self.original {
            self.output = Some(out);
        }
        self.second = decision.second;
        self.margin = Some(decision.margin);
        self.chosen = Some(chosen);
        self
    }

    pub fn confidence(&self, strong_margin: f64) -> Option<TypingDecisionConfidence> {
        self.chosen.as_ref()?;
        Some(classify_typing_confidence(
            self.second.is_some(),
            self.margin,
            strong_margin,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingRuleEvaluation {
    pub id: String,
    pub priority: i32,
    pub enabled: bool,
    pub candidate: Option<TypingCandidate>,
    pub rejected: Option<String>,
}

impl TypingRuleEvaluation {
    pub(crate) const REJECT_DISABLED: &'static str = "disabled";
    pub(crate) const REJECT_NO_CANDIDATE: &'static str = "no candidate";
    pub(crate) const REJECT_UNKNOWN_RULE: &'static str = "unknown rule";
    pub(crate) const REJECT_UNSAFE: &'static str = "unsafe autocorrect candidate";

    pub(crate) fn new(rule: &TypingAssistRuleConfig) -> Self {
        Self {
            id: rule.id.clone(),
            priority: rule.priority,
            enabled: rule.enabled,
            candidate: None,
            rejected: None,
        }
    }

    pub(crate) fn with_candidate(mut self, candidate: TypingCandidate) -> Self {
        self.candidate = Some(candidate);
        self
    }

    pub(crate) fn reject(mut self, reason: &'static str) -> Self {
        self.rejected = Some(reason.to_string());
        self
    }
}
