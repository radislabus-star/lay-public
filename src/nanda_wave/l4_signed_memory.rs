use super::usage_prior::UsagePriorSnapshot;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct L4SignedMemorySignal {
    pub(crate) attraction: f32,
    pub(crate) repulsion: f32,
    pub(crate) signed_weight: f32,
    pub(crate) accepted: u32,
    pub(crate) rejected: u32,
    pub(crate) transition_attraction: f32,
    pub(crate) transition_repulsion: f32,
    pub(crate) transition_attract_count: u32,
    pub(crate) transition_repel_count: u32,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct L4SignedMemoryInput<'a> {
    pub(crate) context: &'a [String],
    pub(crate) source: &'a str,
    pub(crate) operation: &'a str,
    pub(crate) word: &'a str,
    pub(crate) usage: &'a UsagePriorSnapshot,
}

pub(crate) fn l4_signed_memory_signal(input: L4SignedMemoryInput<'_>) -> L4SignedMemorySignal {
    let word_prior = input.usage.word_prior(input.word);
    let context_prior = input.usage.context_word_prior(input.context, input.word);
    let rejected_prior = input.usage.rejected_word_prior(input.word);
    let context_rejected = input
        .usage
        .context_rejected_word_prior(input.context, input.word);
    let transition =
        input
            .usage
            .transition_signal(input.context, input.source, input.operation, input.word);
    let accepted = input.usage.accepted_word_count(input.word);
    let rejected = input.usage.rejected_word_count(input.word);

    let attraction = (word_prior * 0.70
        + context_prior * 1.20
        + transition.attraction * 0.85
        + accepted.min(24) as f32 * 0.014)
        .clamp(0.0, 0.62);
    let repulsion = (rejected_prior * 1.25
        + context_rejected * 1.65
        + transition.repulsion * 0.95
        + rejected.min(24) as f32 * 0.016)
        .clamp(0.0, 0.72);
    let signed_weight = (attraction - repulsion).clamp(-1.0, 1.0);
    let reason = if transition.repulsion > transition.attraction && transition.repel_count > 0 {
        "learned_transition_repels"
    } else if transition.attraction > transition.repulsion && transition.attract_count > 0 {
        "learned_transition_attracts"
    } else if repulsion > attraction && rejected > 0 {
        "learned_state_repels"
    } else if attraction > repulsion && accepted > 0 {
        "learned_state_attracts"
    } else if attraction > 0.0 || repulsion > 0.0 {
        "learned_state_observes"
    } else {
        "learned_state_empty"
    };

    L4SignedMemorySignal {
        attraction,
        repulsion,
        signed_weight,
        accepted,
        rejected,
        transition_attraction: transition.attraction,
        transition_repulsion: transition.repulsion,
        transition_attract_count: transition.attract_count,
        transition_repel_count: transition.repel_count,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::usage_prior;

    fn usage_from_events(text: &str) -> UsagePriorSnapshot {
        usage_prior::snapshot_from_usage_events_for_tests(text)
    }

    #[test]
    fn signed_memory_attracts_accepted_candidate() {
        let usage = usage_from_events(
            r#"{"ts":1,"kind":"accepted_ime","word":"дождь","context":["на","улице","идёт"],"to":"дождь"}
"#,
        );
        let context = ["на", "улице", "идёт"].map(String::from);

        let signal = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "ime",
            operation: "completion",
            word: "дождь",
            usage: &usage,
        });

        assert!(signal.attraction > signal.repulsion);
        assert!(signal.signed_weight > 0.0);
        assert_eq!(signal.reason, "learned_transition_attracts");
    }

    #[test]
    fn signed_memory_repels_corrected_away_candidate() {
        let usage = usage_from_events(
            r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим"}
"#,
        );
        let context = ["мы"].map(String::from);

        let bad = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            word: "отвравим",
            usage: &usage,
        });
        let good = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            word: "отравим",
            usage: &usage,
        });

        assert!(bad.repulsion > bad.attraction);
        assert!(bad.signed_weight < 0.0);
        assert!(good.attraction > good.repulsion);
    }
}
