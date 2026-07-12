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
        if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
            eprintln!(
                "transition-memory origin={} state={} word={} state_specific={} attract={} repel={} signed={:.3}",
                origin.memory_key(),
                state_word,
                word,
                signed.transition_state_specific,
                signed.transition_attract_count,
                signed.transition_repel_count,
                signed.signed_weight
            );
        }
        signed_signal_allows_apply(&signed)
    }
}

fn signed_signal_allows_apply(
    signed: &crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal,
) -> bool {
    if signed.transition_state_specific {
        if signed.transition_attract_count > signed.transition_repel_count {
            return true;
        }
        if signed.transition_repel_count > signed.transition_attract_count {
            return false;
        }
    }
    signed.signed_weight > -0.45
}

#[cfg(test)]
mod tests {
    use super::{signed_signal_allows_apply, TransitionMemory};

    fn signal() -> crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal {
        crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal {
            attraction: 0.10,
            repulsion: 0.62,
            signed_weight: -0.52,
            accepted: 6,
            rejected: 40,
            transition_attraction: 0.10,
            transition_repulsion: 0.0,
            transition_attract_count: 6,
            transition_repel_count: 0,
            transition_state_specific: true,
            reason: "learned_transition_attracts",
            surface_status: "covered",
        }
    }

    #[test]
    fn exact_accept_overrides_global_negative_word_prior() {
        assert!(signed_signal_allows_apply(&signal()));
    }

    #[test]
    fn exact_rejection_blocks_even_with_global_positive_word_prior() {
        let mut signal = signal();
        signal.signed_weight = 0.52;
        signal.transition_attract_count = 0;
        signal.transition_repel_count = 8;

        assert!(!signed_signal_allows_apply(&signal));
    }

    #[test]
    fn empty_candidate_is_not_blocked_by_memory() {
        assert!(TransitionMemory::allows_apply(
            "тест ",
            " ",
            crate::correction_source_contract::CandidateOrigin::DeterministicTypo,
        ));
    }
}
