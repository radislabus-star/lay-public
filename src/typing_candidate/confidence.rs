use super::types::{TypingCandidateDecision, TypingDecisionConfidence};

impl TypingCandidateDecision {
    pub fn confidence(&self, strong_margin: f64) -> TypingDecisionConfidence {
        classify_typing_confidence(self.second.is_some(), Some(self.margin), strong_margin)
    }

    pub fn is_strong(&self, strong_margin: f64) -> bool {
        !matches!(
            self.confidence(strong_margin),
            TypingDecisionConfidence::Weak
        )
    }
}

pub fn classify_typing_confidence(
    has_second_candidate: bool,
    margin: Option<f64>,
    strong_margin: f64,
) -> TypingDecisionConfidence {
    if !has_second_candidate {
        return TypingDecisionConfidence::SingleCandidate;
    }
    if margin.is_some_and(|margin| margin >= strong_margin) {
        TypingDecisionConfidence::Strong
    } else {
        TypingDecisionConfidence::Weak
    }
}
