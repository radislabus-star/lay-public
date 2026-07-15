#[derive(Debug, Clone, Copy)]
struct L3Signal {
    signal: f32,
    rank_bonus: f32,
    decision: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct L4SceneSignal {
    signal: f32,
    rank_bonus: f32,
    action: &'static str,
    reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct L4SignedSignal {
    signal: f32,
    rank_bonus: f32,
    reason: &'static str,
    surface_status: &'static str,
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
    transition_phase_verdict: &'static str,
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
) -> L2WavePeakSignal {
    let score = crate::nanda_wave::l2_wave_peak::score_correction_peak(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
        candidate_count,
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
        transition_phase_verdict: phase.verdict.as_str(),
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
            decision: "not_applicable",
        };
    }
    let Some(report) = evaluate_default_candidate(&event.original, &candidate.replacement) else {
        return L3Signal {
            signal: 0.0,
            rank_bonus: 0.0,
            decision: "no_memory",
        };
    };
    match report.decision {
        L3PhraseGateDecision::Support => {
            let signal = report.score.clamp(0.0, 1.0);
            L3Signal {
                signal,
                rank_bonus: signal * 0.16,
                decision: "support",
            }
        }
        L3PhraseGateDecision::Suppress => L3Signal {
            signal: -0.56,
            rank_bonus: -0.14,
            decision: "suppress",
        },
        L3PhraseGateDecision::Neutral => L3Signal {
            signal: (report.score * 0.20).clamp(0.0, 0.20),
            rank_bonus: 0.0,
            decision: "neutral",
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
        action: scene.allowed_action.as_str(),
        reason: scene.reason,
    }
}

fn l4_signed_signal(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    surface: &str,
) -> L4SignedSignal {
    let mut context = crate::correction_core::normalized_correction_words(&event.original);
    context.pop();
    let word = crate::correction_core::normalized_correction_words(&candidate.replacement)
        .pop()
        .unwrap_or_default();
    if word.is_empty() {
        return L4SignedSignal {
            signal: 0.0,
            rank_bonus: 0.0,
            reason: "learned_state_empty",
            surface_status: "unknown",
            transition_state_specific: false,
            transition_attract_count: 0,
            transition_repel_count: 0,
        };
    }
    let usage = crate::nanda_wave::cached_usage_prior_snapshot();
    let state_id = crate::transition_relation::transition_state_id(&event.original);
    let signed = l4_signed_memory_signal(L4SignedMemoryInput {
        context: &context,
        source: candidate.origin.memory_key(),
        operation: "replacement",
        state_word: &state_id,
        word: &word,
        usage: &usage,
        surface: Some(surface),
    });
    L4SignedSignal {
        signal: signed.signed_weight,
        rank_bonus: signed.signed_weight * 0.12,
        reason: signed.reason,
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
    match action.edit_operator {
        verifier::EditTransitionOperator::BoundaryShift
        | verifier::EditTransitionOperator::BoundaryMergeSplit
        | verifier::EditTransitionOperator::SplitPreviousGluedAndRepairTail => 0.34,
        verifier::EditTransitionOperator::LayoutProjection => 0.28,
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
    }
}

fn micro_to_milli(value: i64) -> i16 {
    (value / 1000).clamp(i16::MIN as i64, i16::MAX as i64) as i16
}
