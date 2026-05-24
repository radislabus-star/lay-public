use crate::typing_candidate::{
    classify_typing_confidence, TypingCandidate, TypingDecisionConfidence,
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
