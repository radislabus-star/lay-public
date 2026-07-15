#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransitionAudit {
    pub operator: Option<String>,
    pub proof: Option<String>,
    pub verified: Option<bool>,
    pub left_context_changed: Option<bool>,
    pub changed_tokens: Option<usize>,
}

impl TransitionAudit {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn proven(
        operator: impl Into<String>,
        proof: impl Into<String>,
        verified: bool,
        left_context_changed: bool,
        changed_tokens: usize,
    ) -> Self {
        Self {
            operator: Some(operator.into()),
            proof: Some(proof.into()),
            verified: Some(verified),
            left_context_changed: Some(left_context_changed),
            changed_tokens: Some(changed_tokens),
        }
    }

    pub fn blocks_apply(&self) -> bool {
        self.verified == Some(false)
            || (self.left_context_changed.unwrap_or(false) && !self.is_verified())
    }

    pub fn is_verified(&self) -> bool {
        self.verified == Some(true)
            && self
                .operator
                .as_deref()
                .is_some_and(|operator| !operator.trim().is_empty())
            && self
                .proof
                .as_deref()
                .is_some_and(|proof| !proof.trim().is_empty())
    }

    pub fn block_reason(&self) -> Option<&'static str> {
        self.blocks_apply()
            .then_some("edit_transition_not_verified")
    }
}
