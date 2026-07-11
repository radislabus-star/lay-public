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
    pub(crate) transition_state_specific: bool,
    pub(crate) reason: &'static str,
    pub(crate) surface_status: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct L4SignedMemoryInput<'a> {
    pub(crate) context: &'a [String],
    pub(crate) source: &'a str,
    pub(crate) operation: &'a str,
    pub(crate) state_word: &'a str,
    pub(crate) word: &'a str,
    pub(crate) usage: &'a UsagePriorSnapshot,
    pub(crate) surface: Option<&'a str>,
}

pub(crate) fn l4_signed_memory_signal(input: L4SignedMemoryInput<'_>) -> L4SignedMemorySignal {
    let readout = input.usage.hot_readout(
        input.context,
        input.source,
        input.operation,
        input.state_word,
        input.word,
    );
    let coverage = input
        .surface
        .map(|surface| input.usage.surface_coverage(surface))
        .unwrap_or_default();
    let surface_status = if coverage.rejected > coverage.accepted {
        "repelled"
    } else if coverage.accepted > 0 {
        "covered"
    } else if coverage.observed > 0 {
        "observed"
    } else {
        "unknown"
    };

    let attraction = (readout.word_prior * 0.70
        + readout.context_prior * 1.20
        + readout.transition.attraction * 0.85
        + readout.accepted_count.min(24) as f32 * 0.014)
        .clamp(0.0, 0.62);
    let repulsion = (readout.rejected_prior * 1.25
        + readout.context_rejected * 1.65
        + readout.transition.repulsion * 0.95
        + readout.rejected_count.min(24) as f32 * 0.016)
        .clamp(0.0, 0.72);
    let signed_weight = (attraction - repulsion).clamp(-1.0, 1.0);
    let reason = if readout.transition.repulsion > readout.transition.attraction
        && readout.transition.repel_count > 0
    {
        "learned_transition_repels"
    } else if readout.transition.attraction > readout.transition.repulsion
        && readout.transition.attract_count > 0
    {
        "learned_transition_attracts"
    } else if repulsion > attraction && readout.rejected_count > 0 {
        "learned_state_repels"
    } else if attraction > repulsion && readout.accepted_count > 0 {
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
        accepted: readout.accepted_count,
        rejected: readout.rejected_count,
        transition_attraction: readout.transition.attraction,
        transition_repulsion: readout.transition.repulsion,
        transition_attract_count: readout.transition.attract_count,
        transition_repel_count: readout.transition.repel_count,
        transition_state_specific: readout.transition.state_specific,
        reason,
        surface_status,
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
            state_word: "д",
            word: "дождь",
            usage: &usage,
            surface: None,
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
            state_word: "отвравим",
            word: "отвравим",
            usage: &usage,
            surface: None,
        });
        let good = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            state_word: "отвравим",
            word: "отравим",
            usage: &usage,
            surface: None,
        });

        assert!(bad.repulsion > bad.attraction);
        assert!(bad.signed_weight < 0.0);
        assert!(good.attraction > good.repulsion);
    }

    #[test]
    fn signed_memory_keeps_context_rejection_out_of_structural_surface_repel() {
        let surface = "op=replacement|source=autocorrect|words=2->2|delta=0|prefix=3-4|edit=1";
        let covered_usage = usage_from_events(&format!(
            "{{\"ts\":1,\"kind\":\"accepted_fix\",\"word\":\"проверить\",\"context\":[\"можно\"],\"from\":\"можно проврить\",\"to\":\"можно проверить\",\"source\":\"autocorrect\",\"operation\":\"replacement\",\"surface\":\"{surface}\"}}\n"
        ));
        let repelled_usage = usage_from_events(&format!(
            "{{\"ts\":1,\"kind\":\"rejected_candidate\",\"word\":\"проврить\",\"context\":[\"можно\"],\"from\":\"можно проверить\",\"to\":\"можно проврить\",\"source\":\"autocorrect\",\"operation\":\"replacement\",\"surface\":\"{surface}\"}}\n"
        ));
        let context = ["можно".to_string()];

        let covered = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            state_word: "проврить",
            word: "проверить",
            usage: &covered_usage,
            surface: Some(surface),
        });
        let repelled = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            state_word: "проверить",
            word: "проврить",
            usage: &repelled_usage,
            surface: Some(surface),
        });

        assert_eq!(covered.surface_status, "covered");
        assert_eq!(repelled.surface_status, "observed");
    }
}
