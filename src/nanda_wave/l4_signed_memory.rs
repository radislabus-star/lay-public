use super::usage_prior::{UsageHotReadout, UsagePriorSnapshot, UsageSurfaceCoverage};

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
    pub(crate) phase_witness_milli: i16,
    pub(crate) phase_witness_supported: bool,
    pub(crate) phase_positive_centers: u8,
    pub(crate) phase_negative_centers: u8,
    pub(crate) reason: L4SignedMemoryReason,
    pub(crate) surface_status: L4SurfaceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4SignedMemoryReason {
    TransitionRepels,
    TransitionAttracts,
    StateRepels,
    StateAttracts,
    StateObserved,
    Empty,
}

impl L4SignedMemoryReason {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::TransitionRepels => "learned_transition_repels",
            Self::TransitionAttracts => "learned_transition_attracts",
            Self::StateRepels => "learned_state_repels",
            Self::StateAttracts => "learned_state_attracts",
            Self::StateObserved => "learned_state_observes",
            Self::Empty => "learned_state_empty",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4SurfaceStatus {
    Repelled,
    Covered,
    Observed,
    Unknown,
}

impl L4SurfaceStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Repelled => "repelled",
            Self::Covered => "covered",
            Self::Observed => "observed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct L4SignedMemoryInput<'a> {
    pub(crate) context: &'a [String],
    pub(crate) source: &'a str,
    pub(crate) operation: &'a str,
    pub(crate) state_word: &'a str,
    pub(crate) candidate_text: &'a str,
    pub(crate) usage: &'a UsagePriorSnapshot,
    pub(crate) surface: Option<&'a str>,
}

pub(crate) fn l4_signed_memory_signal(input: L4SignedMemoryInput<'_>) -> L4SignedMemorySignal {
    let readout = input.usage.hot_readout(
        input.context,
        input.source,
        input.operation,
        input.state_word,
        input.candidate_text,
    );
    let coverage = input
        .surface
        .map(|surface| input.usage.surface_coverage(surface))
        .unwrap_or_default();
    let phase = input
        .surface
        .map(|surface| input.usage.phase_witness(surface))
        .unwrap_or_default();
    l4_signed_memory_signal_from_parts(readout, coverage, phase)
}

pub(crate) fn l4_signed_memory_signal_from_readout(
    readout: UsageHotReadout,
    coverage: UsageSurfaceCoverage,
) -> L4SignedMemorySignal {
    l4_signed_memory_signal_from_parts(readout, coverage, Default::default())
}

fn l4_signed_memory_signal_from_parts(
    readout: UsageHotReadout,
    coverage: UsageSurfaceCoverage,
    phase: super::l4_phase_witness::L4PhaseWitnessReadout,
) -> L4SignedMemorySignal {
    let surface_evidence = coverage.accepted.saturating_add(coverage.rejected) as f32;
    let surface_signed_confidence = if surface_evidence > 0.0 {
        (coverage.accepted as f32 - coverage.rejected as f32) / (surface_evidence + 8.0)
    } else {
        0.0
    };
    let surface_status = if surface_signed_confidence < -0.5 && !readout.transition.state_specific {
        L4SurfaceStatus::Repelled
    } else if coverage.accepted > 0 {
        L4SurfaceStatus::Covered
    } else if coverage.observed > 0 {
        L4SurfaceStatus::Observed
    } else {
        L4SurfaceStatus::Unknown
    };

    let positive_evidence = readout.accepted_count as f32
        + readout.transition.attract_count as f32
        + (readout.word_prior + readout.context_prior + readout.transition.attraction) * 4.0;
    let negative_evidence = readout.rejected_count as f32
        + readout.transition.repel_count as f32
        + (readout.rejected_prior + readout.context_rejected + readout.transition.repulsion) * 4.0;
    let observed = positive_evidence + negative_evidence;
    let posterior = (positive_evidence + 1.0) / (observed + 2.0);
    let confidence = observed / (observed + 4.0);
    let signed_weight = ((posterior * 2.0 - 1.0) * confidence).clamp(-1.0, 1.0);
    let attraction = signed_weight.max(0.0);
    let repulsion = (-signed_weight).max(0.0);
    let reason = if readout.transition.repulsion > readout.transition.attraction
        && readout.transition.repel_count > 0
    {
        L4SignedMemoryReason::TransitionRepels
    } else if readout.transition.attraction > readout.transition.repulsion
        && readout.transition.attract_count > 0
    {
        L4SignedMemoryReason::TransitionAttracts
    } else if repulsion > attraction && readout.rejected_count > 0 {
        L4SignedMemoryReason::StateRepels
    } else if attraction > repulsion && readout.accepted_count > 0 {
        L4SignedMemoryReason::StateAttracts
    } else if attraction > 0.0 || repulsion > 0.0 {
        L4SignedMemoryReason::StateObserved
    } else {
        L4SignedMemoryReason::Empty
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
        phase_witness_milli: crate::text_metrics::score_to_milli(phase.margin),
        phase_witness_supported: phase.supported,
        phase_positive_centers: phase.positive_centers,
        phase_negative_centers: phase.negative_centers,
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
            candidate_text: "дождь",
            usage: &usage,
            surface: None,
        });

        assert!(signal.attraction > signal.repulsion);
        assert!(signal.signed_weight > 0.0);
        assert_eq!(signal.reason, L4SignedMemoryReason::TransitionAttracts);
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
            candidate_text: "отвравим",
            usage: &usage,
            surface: None,
        });
        let good = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            state_word: "отвравим",
            candidate_text: "отравим",
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
            candidate_text: "проверить",
            usage: &covered_usage,
            surface: Some(surface),
        });
        let repelled = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &context,
            source: "autocorrect",
            operation: "replacement",
            state_word: "проверить",
            candidate_text: "проврить",
            usage: &repelled_usage,
            surface: Some(surface),
        });

        assert_eq!(covered.surface_status, L4SurfaceStatus::Covered);
        assert_eq!(repelled.surface_status, L4SurfaceStatus::Observed);
    }

    #[test]
    fn signed_memory_addresses_complete_multiword_transition() {
        let usage = usage_from_events(
            r#"{"ts":1,"kind":"rejected_candidate","word":"слов","from":"мыслов","to":"мы слов","source":"typing-assist","operation":"boundary"}
"#,
        );
        let state = crate::transition_relation::transition_state_id("мыслов");

        let split = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &[],
            source: "boundary",
            operation: "replacement",
            state_word: &state,
            candidate_text: "мы слов",
            usage: &usage,
            surface: None,
        });
        let unrelated_tail = l4_signed_memory_signal(L4SignedMemoryInput {
            context: &[],
            source: "boundary",
            operation: "replacement",
            state_word: &state,
            candidate_text: "слов",
            usage: &usage,
            surface: None,
        });

        assert!(split.transition_state_specific);
        assert!(split.transition_repulsion > split.transition_attraction);
        assert_eq!(split.reason, L4SignedMemoryReason::TransitionRepels);
        assert_eq!(unrelated_tail.transition_repel_count, 0);
    }
}
