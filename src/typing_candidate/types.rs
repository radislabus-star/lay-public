#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingCandidateFamily {
    Exact,
    Visual,
    Layout,
    Structural,
    Typo,
    Cleanup,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingCandidateScore {
    pub total: f64,
    pub family: TypingCandidateFamily,
    pub family_weight: f64,
    pub language_delta: f64,
    pub structure_bonus: f64,
    pub edit_penalty: f64,
    pub intervention_penalty: f64,
    pub priority_bonus: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingCandidate {
    pub rule_id: String,
    pub priority: i32,
    pub replacement: String,
    pub score: TypingCandidateScore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingCandidateDecision {
    pub best: TypingCandidate,
    pub second: Option<TypingCandidate>,
    pub margin: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingDecisionConfidence {
    SingleCandidate,
    Strong,
    Weak,
}
