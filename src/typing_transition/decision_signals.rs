#[derive(Debug, Clone, Copy)]
struct L3Signal {
    signal: f32,
    rank_bonus: f32,
    decision: L3ContextDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L3ContextDisposition {
    NotApplicable,
    Unavailable,
    Neutral,
    Support,
    Suppress,
}

impl L3ContextDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Unavailable => "no_memory",
            Self::Neutral => "neutral",
            Self::Support => "support",
            Self::Suppress => "suppress",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct L4SceneSignal {
    signal: f32,
    rank_bonus: f32,
    action: L4AllowedAction,
    reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct L4SignedSignal {
    signal: f32,
    rank_bonus: f32,
    reason: &'static str,
    surface_status: crate::nanda_wave::l4_signed_memory::L4SurfaceStatus,
    transition_state_specific: bool,
    transition_attract_count: u32,
    transition_repel_count: u32,
}

#[derive(Debug, Clone, Copy)]
struct L2WavePeakSignal {
    signal: f32,
    rank_bonus: f32,
    positive_milli: i16,
    negative_milli: i16,
    uncertainty_milli: i16,
    reason: &'static str,
    transition_phase_milli: i16,
    transition_phase_threshold_milli: i16,
    transition_phase_verdict: crate::nanda_wave::PhaseVerdict,
    transition_phase_package_loaded: bool,
    transition_phase_operator_present: bool,
    transition_phase_operator_promoted: bool,
    transition_phase_positive_centers: u8,
    transition_phase_anti_centers: u8,
    transition_phase_surfaces: u32,
}

fn l2_wave_peak_signal(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    phase: crate::nanda_wave::PhaseReadout,
    usage: &crate::nanda_wave::UsagePriorSnapshot,
) -> L2WavePeakSignal {
    let score = crate::nanda_wave::l2_wave_peak::score_correction_peak_with_usage(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
        candidate_count,
        usage,
    );
    L2WavePeakSignal {
        signal: score.signal,
        rank_bonus: score.rank_bonus,
        positive_milli: score.positive_milli,
        negative_milli: score.negative_milli,
        uncertainty_milli: score.uncertainty_milli,
        reason: score.reason,
        transition_phase_milli: micro_to_milli(phase.margin_micro),
        transition_phase_threshold_milli: micro_to_milli(phase.threshold_micro),
        transition_phase_verdict: phase.verdict,
        transition_phase_package_loaded: phase.package_loaded,
        transition_phase_operator_present: phase.operator_present,
        transition_phase_operator_promoted: phase.operator_promoted,
        transition_phase_positive_centers: phase.positive_centers,
        transition_phase_anti_centers: phase.anti_centers,
        transition_phase_surfaces: phase.covered_surfaces,
    }
}

fn l3_phrase_signal(event: &TypingErrorEvent, candidate: &UnifiedCorrectionCandidate) -> L3Signal {
    if !l3_phrase_signal_observes(candidate.error_class) {
        return L3Signal {
            signal: 0.0,
            rank_bonus: 0.0,
            decision: L3ContextDisposition::NotApplicable,
        };
    }
    let Some(report) = evaluate_default_candidate(&event.original, &candidate.replacement) else {
        return L3Signal {
            signal: 0.0,
            rank_bonus: 0.0,
            decision: L3ContextDisposition::Unavailable,
        };
    };
    match report.decision {
        L3PhraseGateDecision::Support => {
            let signal = report.score.clamp(0.0, 1.0);
            L3Signal {
                signal,
                rank_bonus: signal * 0.16,
                decision: L3ContextDisposition::Support,
            }
        }
        L3PhraseGateDecision::Suppress => L3Signal {
            signal: -0.56,
            rank_bonus: -0.14,
            decision: L3ContextDisposition::Suppress,
        },
        L3PhraseGateDecision::Neutral => L3Signal {
            signal: (report.score * 0.20).clamp(0.0, 0.20),
            rank_bonus: 0.0,
            decision: L3ContextDisposition::Neutral,
        },
    }
}

fn l3_phrase_signal_observes(error_class: TypingErrorClass) -> bool {
    !matches!(
        error_class,
        TypingErrorClass::CompletionOnly
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::Unknown
    )
}

fn l4_scene_signal(event: &TypingErrorEvent, candidate_count: usize) -> L4SceneSignal {
    let scene = derive_l4_scene_state(L4SceneStateInput {
        context_prefix: &event.core,
        current_word: &event.current_word,
        candidate_count,
    });
    let signal = match scene.allowed_action {
        L4AllowedAction::Suggest => scene.confidence,
        L4AllowedAction::Wait => -scene.confidence * 0.50,
        L4AllowedAction::Block => -scene.confidence,
    };
    L4SceneSignal {
        signal,
        rank_bonus: signal * 0.06,
        action: scene.allowed_action,
        reason: scene.reason,
    }
}

fn l4_signed_memory_readout(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    surface: &str,
    usage: &crate::nanda_wave::UsagePriorSnapshot,
) -> crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal {
    let context = crate::typing_memory::transition_context_words(
        &event.original,
        &candidate.replacement,
    );
    let transition_target =
        crate::typing_memory::transition_target_text(&event.original, &candidate.replacement);
    if transition_target.is_empty() {
        return crate::nanda_wave::l4_signed_memory::l4_signed_memory_signal_from_readout(
            crate::nanda_wave::usage_prior::UsageHotReadout::default(),
            crate::nanda_wave::usage_prior::UsageSurfaceCoverage::default(),
        );
    }
    let state_id = crate::transition_relation::transition_state_id(&event.original);
    l4_signed_memory_signal(L4SignedMemoryInput {
        context: &context,
        source: candidate.origin.memory_key(),
        operation: "replacement",
        state_word: &state_id,
        candidate_text: &transition_target,
        usage,
        surface: Some(surface),
    })
}

fn l4_signed_signal_from_memory(
    signed: &crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal,
) -> L4SignedSignal {
    L4SignedSignal {
        signal: signed.signed_weight,
        rank_bonus: signed.signed_weight * 0.12,
        reason: signed.reason.as_str(),
        surface_status: signed.surface_status,
        transition_state_specific: signed.transition_state_specific,
        transition_attract_count: signed.transition_attract_count,
        transition_repel_count: signed.transition_repel_count,
    }
}

fn transition_rank_bonus(
    action: &action::CorrectionActionOperatorReport,
    candidate: &UnifiedCorrectionCandidate,
) -> f32 {
    if !action.verifier_passed {
        return -0.20;
    }
    let operator_bonus = match action.edit_operator {
        verifier::EditTransitionOperator::BoundaryShift
        | verifier::EditTransitionOperator::BoundaryMergeSplit
        | verifier::EditTransitionOperator::SplitPreviousGluedAndRepairTail => 0.34,
        verifier::EditTransitionOperator::LayoutProjection => 0.08,
        verifier::EditTransitionOperator::PhraseTokenRepair => 0.16,
        verifier::EditTransitionOperator::ReplaceCurrentWord => {
            if candidate
                .has_origin(crate::candidate_contract::CandidateOrigin::DeterministicTypo)
            {
                0.08
            } else if candidate
                .has_origin(crate::candidate_contract::CandidateOrigin::L2Surface)
            {
                -0.08
            } else {
                0.0
            }
        }
        verifier::EditTransitionOperator::Completion
        | verifier::EditTransitionOperator::Protected
        | verifier::EditTransitionOperator::Unknown
        | verifier::EditTransitionOperator::VisibleTail
        | verifier::EditTransitionOperator::DecoderTail
        | verifier::EditTransitionOperator::ManualReplace
        | verifier::EditTransitionOperator::Undo
        | verifier::EditTransitionOperator::EnterAutocorrect
        | verifier::EditTransitionOperator::NativeReplace => 0.0,
    };
    // A composed layout+typo projection consumes two operators. It must beat
    // a one-step center on evidence, independently of its physical edit shape.
    let composition_cost = if candidate.origin
        == crate::candidate_contract::CandidateOrigin::LayoutThenTypo
    {
        0.12
    } else {
        0.0
    };
    operator_bonus - composition_cost
}

fn micro_to_milli(value: i64) -> i16 {
    (value / 1000).clamp(i16::MIN as i64, i16::MAX as i64) as i16
}
