use super::*;

/// Opaque proof that `TransitionDecisionCore` selected this exact semantic
/// transition. Logs can describe the transition, but cannot manufacture this
/// receipt or use a serialized trace as apply authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecisionTransitionReceipt {
    original: String,
    replacement: String,
    transition: TransitionAudit,
}

impl DecisionTransitionReceipt {
    pub(super) fn issue(
        original: String,
        replacement: String,
        transition: TransitionAudit,
    ) -> Self {
        Self {
            original,
            replacement,
            transition,
        }
    }

    /// A visible-tail verifier may issue this receipt only after it has bound
    /// the exact observed snapshot. It is not candidate-selection authority.
    pub(crate) fn for_visible_tail(
        original: String,
        replacement: String,
        transition: TransitionAudit,
    ) -> Self {
        Self::issue(original, replacement, transition)
    }

    pub(super) fn from_selected_candidate(
        event: &TypingErrorEvent,
        candidate: &UnifiedCorrectionCandidate,
        evaluation: &CandidateDecisionEvaluation,
    ) -> Self {
        let action = evaluation.action;
        Self::issue(
            event.original.clone(),
            candidate.replacement.clone(),
            TransitionAudit::proven(
                action.edit_operator,
                action.edit_proof.into(),
                action.verifier_passed,
                action.left_context_changed,
                action.changed_tokens,
            ),
        )
    }

    pub(crate) fn projected_transition(
        &self,
        from_text: &str,
        to_text: &str,
    ) -> Option<TransitionAudit> {
        if !self.transition.is_verified()
            || !same_transition_projection(&self.original, &self.replacement, from_text, to_text)
        {
            return None;
        }
        Some(self.transition.clone())
    }

    pub(crate) fn diagnostic_transition(&self) -> TransitionAudit {
        self.transition.clone()
    }
}

fn same_transition_projection(
    original: &str,
    replacement: &str,
    from_text: &str,
    to_text: &str,
) -> bool {
    if original == from_text && replacement == to_text {
        return true;
    }
    if let Some(stable_suffix) = from_text.strip_prefix(original) {
        return to_text
            .strip_prefix(replacement)
            .is_some_and(|projected_suffix| projected_suffix == stable_suffix);
    }
    let Some(original_prefix) = original.strip_suffix(from_text) else {
        return false;
    };
    replacement
        .strip_suffix(to_text)
        .is_some_and(|replacement_prefix| replacement_prefix == original_prefix)
}

#[cfg(test)]
mod tests {
    use super::same_transition_projection;

    #[test]
    fn transition_receipt_projects_over_unchanged_right_context() {
        assert!(same_transition_projection(
            "постаивм ",
            "поставим ",
            "постаивм хвост",
            "поставим хвост",
        ));
        assert!(!same_transition_projection(
            "постаивм ",
            "поставим ",
            "постаивм хвост",
            "поставим другой",
        ));
    }
}
