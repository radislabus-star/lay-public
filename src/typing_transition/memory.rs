//! L4 signed transition memory admission.
//!
//! This layer is intentionally narrow on the hot path. It answers whether past
//! accepted/rejected experience blocks an otherwise valid transition.

pub(crate) struct TransitionMemory;

impl TransitionMemory {
    pub(crate) fn allows_apply(
        original: &str,
        replacement: &str,
        origin: crate::correction_source_contract::CandidateOrigin,
    ) -> bool {
        let mut context = crate::correction_core::normalized_correction_words(original);
        context.pop();
        let word = crate::correction_core::normalized_correction_words(replacement)
            .pop()
            .unwrap_or_default();
        let state_word = crate::transition_relation::transition_state_id(original);
        if word.is_empty() {
            return true;
        }
        let usage = crate::nanda_wave::cached_usage_prior_snapshot();
        let signed = crate::nanda_wave::l4_signed_memory::l4_signed_memory_signal(
            crate::nanda_wave::l4_signed_memory::L4SignedMemoryInput {
                context: &context,
                source: origin.memory_key(),
                operation: "replacement",
                state_word: &state_word,
                word: &word,
                usage: &usage,
                surface: None,
            },
        );
        let exact_transition_rejected = signed.transition_state_specific
            && signed.transition_repel_count > signed.transition_attract_count;
        !exact_transition_rejected && signed.signed_weight > -0.45
    }
}

#[cfg(test)]
mod tests {
    use super::TransitionMemory;

    #[test]
    fn empty_candidate_is_not_blocked_by_memory() {
        assert!(TransitionMemory::allows_apply(
            "тест ",
            " ",
            crate::correction_source_contract::CandidateOrigin::DeterministicTypo,
        ));
    }
}
