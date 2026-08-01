#[derive(Debug, Clone, Copy)]
struct L3Signal {
    signal: f32,
    rank_energy: f32,
    decision: L3ContextDisposition,
    relation_class: u64,
    pairwise_certified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L3ContextDisposition {
    NotApplicable,
    Neutral,
    Support,
    Suppress,
}

impl L3ContextDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Neutral => "neutral",
            Self::Support => "support",
            Self::Suppress => "suppress",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct L4SignedSignal {
    signal: f32,
    rank_energy: f32,
    reason: &'static str,
    surface_status: crate::nanda_wave::l4_signed_memory::L4SurfaceStatus,
    transition_state_specific: bool,
    transition_attract_count: u32,
    transition_repel_count: u32,
    phase_witness_milli: i16,
    phase_witness_supported: bool,
    phase_positive_centers: u8,
    phase_negative_centers: u8,
}

#[derive(Debug, Clone, Copy)]
struct L2WavePeakSignal {
    signal: f32,
    rank_energy: f32,
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
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    phase: crate::nanda_wave::PhaseReadout,
    usage: &crate::nanda_wave::UsagePriorSnapshot,
    peak_context: &crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext,
) -> L2WavePeakSignal {
    let score = crate::nanda_wave::l2_wave_peak::score_correction_peak_with_prepared_usage(
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
        candidate_count,
        usage,
        peak_context,
    );
    L2WavePeakSignal {
        signal: score.signal,
        rank_energy: score.rank_bonus,
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

fn l3_phrase_signal(
    error_class: TypingErrorClass,
    report: Option<&crate::nanda_wave::l3_phrase_gate::L3PhraseGateReport>,
) -> L3Signal {
    if !l3_phrase_signal_observes(error_class) {
        return L3Signal {
            signal: 0.0,
            rank_energy: 0.0,
            decision: L3ContextDisposition::NotApplicable,
            relation_class: 0,
            pairwise_certified: false,
        };
    }
    let Some(report) = report else {
        return L3Signal {
            signal: 0.0,
            rank_energy: 0.0,
            decision: L3ContextDisposition::NotApplicable,
            relation_class: 0,
            pairwise_certified: false,
        };
    };
    match report.decision {
        L3PhraseGateDecision::Support => {
            let signal = report.score.clamp(0.0, 1.0);
            L3Signal {
                signal,
                rank_energy: report.rank_energy,
                decision: L3ContextDisposition::Support,
                relation_class: report.relation_class,
                pairwise_certified: report.pairwise_certified,
            }
        }
        L3PhraseGateDecision::Suppress => L3Signal {
            signal: report.score.min(0.0),
            rank_energy: report.rank_energy,
            decision: L3ContextDisposition::Suppress,
            relation_class: report.relation_class,
            pairwise_certified: false,
        },
        L3PhraseGateDecision::Neutral => L3Signal {
            signal: (report.score * 0.20).clamp(0.0, 0.20),
            rank_energy: 0.0,
            decision: L3ContextDisposition::Neutral,
            relation_class: report.relation_class,
            pairwise_certified: false,
        },
    }
}

fn l3_phrase_signal_observes(error_class: TypingErrorClass) -> bool {
    !matches!(
        error_class,
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken
    )
}

fn l4_signed_memory_readout(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    surface: &str,
    usage: &crate::nanda_wave::UsagePriorSnapshot,
) -> crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal {
    let context =
        crate::typing_memory::transition_context_words(&event.original, &candidate.replacement);
    if candidate.replacement.trim().is_empty() {
        return crate::nanda_wave::l4_signed_memory::l4_signed_memory_signal_from_readout(
            crate::nanda_wave::usage_prior::UsageHotReadout::default(),
            crate::nanda_wave::usage_prior::UsageSurfaceCoverage::default(),
        );
    }
    let state_id = crate::transition_relation::signed_memory_state_id(&event.original);
    let operator = crate::typing_memory::transition_learning_key(
        &event.original,
        &candidate.replacement,
        "replacement",
    );
    l4_signed_memory_signal(L4SignedMemoryInput {
        context: &context,
        source: candidate.origin.memory_key(),
        operation: &operator,
        state_word: &state_id,
        candidate_text: &candidate.replacement,
        usage,
        surface: Some(surface),
    })
}

fn l4_signed_signal_from_memory(
    signed: &crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal,
) -> L4SignedSignal {
    L4SignedSignal {
        signal: signed.signed_weight,
        rank_energy: signed.signed_weight * 0.12,
        reason: signed.reason.as_str(),
        surface_status: signed.surface_status,
        transition_state_specific: signed.transition_state_specific,
        transition_attract_count: signed.transition_attract_count,
        transition_repel_count: signed.transition_repel_count,
        phase_witness_milli: signed.phase_witness_milli,
        phase_witness_supported: signed.phase_witness_supported,
        phase_positive_centers: signed.phase_positive_centers,
        phase_negative_centers: signed.phase_negative_centers,
    }
}

fn l4_cross_scene_shadow_readout(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    relation: &TransitionRelationAtoms,
    l3: L3Signal,
    l2: L2WavePeakSignal,
) -> crate::nanda_wave::l4_cross_scene::L4CrossSceneReadout {
    let identity = crate::typing_memory::TypingTransitionIdentity::observed(
        &event.original,
        &candidate.replacement,
        "replacement",
    );
    let context = crate::typing_memory::transition_context_words(
        &event.original,
        &candidate.replacement,
    );
    let l2_signal = if l2.signal > 0.0 && l2.positive_milli > l2.negative_milli {
        crate::nanda_wave::l4_cross_scene::L4CrossSceneL2Signal::Support
    } else if l2.signal < 0.0 && l2.negative_milli > l2.positive_milli {
        crate::nanda_wave::l4_cross_scene::L4CrossSceneL2Signal::Repel
    } else {
        crate::nanda_wave::l4_cross_scene::L4CrossSceneL2Signal::Unknown
    };
    let profile = crate::nanda_wave::l4_cross_scene::L4CrossSceneProfileKey::new(
        identity.operator,
        identity.layout_direction,
        identity.layout_scope,
    );
    let candidate_relation_id =
        crate::nanda_wave::l4_cross_scene::candidate_relation_id(relation.atoms());
    let keep_relation_id = crate::nanda_wave::l4_cross_scene::keep_relation_id();
    let context_signal = crate::nanda_wave::l4_cross_scene::context_signal_from_text(
        &context,
        &candidate.replacement,
    );
    let relation_class = if l3.relation_class == 0 {
        crate::nanda_wave::l4_cross_scene::relation_class_from_context(
            &context,
            &candidate.replacement,
        )
    } else {
        l3.relation_class
    };
    crate::nanda_wave::l4_cross_scene::shadow_readout(
        crate::nanda_wave::l4_cross_scene::L4CrossSceneInput {
            profile,
            context: &context,
            from_text: &event.original,
            to_text: &candidate.replacement,
            relation_atoms: relation.atoms(),
            candidate_relation_id,
            keep_relation_id,
            l3_relation_class: relation_class,
            context_signal,
            l2_signal,
        },
    )
}

fn transition_interference_readout(
    l2: L2WavePeakSignal,
    phase: crate::nanda_wave::PhaseReadout,
    l3: L3Signal,
    l4_signed: L4SignedSignal,
    phase_competition: Option<f32>,
) -> interference::TransitionInterferenceReadout {
    interference::read_transition_interference(interference::TransitionInterferenceInput {
        l2_rank_energy: l2.rank_energy,
        l2_uncertainty: l2.uncertainty_milli as f32 / 1_000.0,
        phase,
        phase_competition,
        l3_rank_energy: l3.rank_energy,
        l4_signed_rank_energy: l4_signed.rank_energy,
    })
}

fn settle_transition_interference(
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &mut [CandidateDecisionEvaluation],
    policy: TransitionDecisionPolicy,
) {
    // Phase evidence is a contrastive field over the complete eligible lattice.
    // It only redistributes existing L2 energy; admission remains independent.
    let strengths = candidates
        .iter()
        .zip(evaluations.iter())
        .filter_map(|(candidate, evaluation)| {
            (policy.l2_phase_apply && candidate.gate.action == CandidateGateAction::Eligible)
                .then(|| phase_lattice_strength(&evaluation.signals))
                .flatten()
        })
        .collect::<Vec<_>>();
    let phase_bounds = strengths
        .iter()
        .copied()
        .min_by(f32::total_cmp)
        .zip(strengths.iter().copied().max_by(f32::total_cmp))
        .filter(|(minimum, maximum)| minimum < maximum);

    for (candidate, evaluation) in candidates.iter().zip(evaluations) {
        let phase_competition = phase_bounds.and_then(|(minimum, maximum)| {
            (candidate.gate.action == CandidateGateAction::Eligible)
                .then(|| phase_lattice_strength(&evaluation.signals))
                .flatten()
                .map(|strength| {
                    let position = (strength - minimum) / (maximum - minimum);
                    position.mul_add(2.0, -1.0)
                })
        });
        let phase = phase_readout_from_signals(&evaluation.signals);
        let field =
            interference::read_transition_interference(interference::TransitionInterferenceInput {
                l2_rank_energy: evaluation.signals.l2_rank_energy,
                l2_uncertainty: evaluation.signals.l2_wave_peak_uncertainty_milli as f32 / 1_000.0,
                phase,
                phase_competition,
                l3_rank_energy: evaluation.signals.l3_rank_energy,
                l4_signed_rank_energy: evaluation.signals.l4_signed_rank_energy,
            });
        let signals = &mut evaluation.signals;
        signals.rank_score = signals.non_field_rank_score + field.signal;
        signals.rank_milli = score_to_milli(signals.rank_score);
        signals.transition_field_milli = score_to_milli(field.signal);
        signals.transition_field_attraction_milli = score_to_milli(field.attraction);
        signals.transition_field_repulsion_milli = score_to_milli(field.repulsion);
        signals.transition_field_uncertainty_milli = score_to_milli(field.uncertainty);
        signals.transition_field_phase_competition_milli = score_to_milli(field.phase_competition);
    }
}

fn settle_l4_hidden_state(
    event: &TypingErrorEvent,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &mut [CandidateDecisionEvaluation],
) {
    let inputs = candidates
        .iter()
        .zip(evaluations.iter())
        .map(|(candidate, evaluation)| L4HiddenCandidateInput {
            predicted_state: predicted_state_id(
                crate::nanda_wave::phase_field::hash_text(&event.original),
                evaluation.action.operator.as_str(),
                &candidate.replacement,
            ),
            relation_class: evaluation.signals.l3_relation_class,
            operator_class: crate::nanda_wave::phase_field::hash_text(
                evaluation.action.operator.as_str(),
            ),
            verifier_passed: evaluation.action.verifier_passed,
            rank_milli: evaluation.signals.rank_milli,
            context_support: evaluation.signals.l3_phrase_decision == L3ContextDisposition::Support
                || (evaluation.signals.l2_transition_phase_verdict
                    == crate::nanda_wave::PhaseVerdict::Support
                    && evaluation.signals.l2_lexical_phase_competition_ready)
                || (evaluation.signals.l2_wave_peak_positive_milli
                    > evaluation.signals.l2_wave_peak_negative_milli
                    && evaluation.signals.l2_wave_peak_uncertainty_milli
                        < evaluation
                            .signals
                            .l2_wave_peak_positive_milli
                            .saturating_sub(evaluation.signals.l2_wave_peak_negative_milli)),
            pairwise_context_witness: evaluation.signals.l3_pairwise_certified,
            eligible: candidate.gate.action == CandidateGateAction::Eligible,
            witness_attract: evaluation.signals.l4_transition_attract_count,
            witness_repel: evaluation.signals.l4_transition_repel_count,
            witness_state_specific: evaluation.signals.l4_transition_state_specific,
            phase_witness_milli: evaluation.signals.l4_phase_witness_milli,
            phase_witness_supported: evaluation.signals.l4_phase_witness_supported,
            operator_consensus_witness: verified_operator_consensus_witness(candidate, evaluation),
        })
        .collect::<Vec<_>>();
    let readouts = estimate_hidden_typing_state(&inputs);
    for (evaluation, readout) in evaluations.iter_mut().zip(readouts) {
        let signals = &mut evaluation.signals;
        signals.l4_hidden_disposition = readout.disposition;
        signals.l4_hidden_semantic_classes = readout.semantic_classes;
        signals.l4_hidden_unresolved_classes = readout.unresolved_classes;
        signals.l4_hidden_selected_class = readout.selected_class;
        signals.l4_hidden_class_margin_milli = readout.class_margin_milli;
        signals.l4_hidden_witness_count = readout.witness_count;
        signals.l4_hidden_ambiguity_authoritative = readout.ambiguity_authoritative;
        signals.l4_hidden_selected_witnessed = readout.selected_witnessed;
        signals.l4_hidden_plan_commitment = readout.witness_plan_commitment;
        signals.l4_hidden_receipts = readout.witness_receipts;
        signals.l4_hidden_probe = readout.witness_probe;
        signals.l4_hidden_certificate_valid = readout.certificate_valid;
        signals.l4_scene_milli = match readout.disposition {
            L4HiddenDisposition::Resolved | L4HiddenDisposition::Witnessed => {
                readout.class_margin_milli.max(1)
            }
            L4HiddenDisposition::Rejected => -readout.class_margin_milli.abs().max(1),
            L4HiddenDisposition::Ambiguous | L4HiddenDisposition::Unobserved => 0,
        };
        signals.l4_scene_action = match readout.disposition {
            L4HiddenDisposition::Resolved | L4HiddenDisposition::Witnessed => {
                L4AllowedAction::Suggest
            }
            L4HiddenDisposition::Rejected => L4AllowedAction::Block,
            L4HiddenDisposition::Ambiguous | L4HiddenDisposition::Unobserved => {
                L4AllowedAction::Wait
            }
        };
        signals.l4_scene_reason = readout.disposition.as_str();
    }
}

fn verified_operator_consensus_witness(
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    let action = evaluation.action;
    let exact_transposition = verified_mass_preserving_transposition(candidate, evaluation);
    let exact_l2_transition = verified_mass_preserving_l2_transition(candidate, evaluation);
    let canonical_local_field_evidence = candidate.has_source_id("CanonicalL2FieldReadout");
    let independent_operator_evidence = (candidate
        .has_origin(crate::candidate_contract::CandidateOrigin::DeterministicTypo)
        && candidate.has_origin(crate::candidate_contract::CandidateOrigin::L2Surface))
        || canonical_local_field_evidence
        || exact_l2_transition;
    let learned_field_evidence = exact_transposition
        || strong_l2_wave_peak_support(&evaluation.signals)
        || (evaluation.signals.l2_transition_phase_verdict
            == crate::nanda_wave::PhaseVerdict::Support
            && evaluation.signals.l2_lexical_phase_competition_ready);
    action.verifier_passed
        && !action.left_context_changed
        && action.changed_tokens == 1
        && is_precise_lexical_operator(action.operator)
        && independent_operator_evidence
        && learned_field_evidence
}

fn verified_mass_preserving_transposition(
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    candidate.error_class == TypingErrorClass::AdjacentTransposition
        && evaluation.action.verifier_passed
        && evaluation.action.edit_operator == verifier::EditTransitionOperator::ReplaceCurrentWord
        && evaluation.explanation.edit_shape == "transpose_adjacent"
        && evaluation.explanation.operator_fit_milli == 1000
        && strong_l2_wave_peak_support(&evaluation.signals)
}

fn verified_mass_preserving_l2_transition(
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    candidate.origin.source_role() == CorrectionSourceRole::L2Surface
        && verified_mass_preserving_transposition(candidate, evaluation)
}

fn strong_l2_wave_peak_support(signals: &CandidateDecisionSignals) -> bool {
    if signals.l2_transition_phase_operator_promoted
        && signals.l2_transition_phase_verdict == crate::nanda_wave::PhaseVerdict::Repel
        && signals.l2_transition_phase_milli < 0
    {
        return false;
    }
    signals.l2_wave_peak_milli >= calibration::CURRENT.l2_peak_milli
        && signals.l2_wave_peak_uncertainty_milli <= calibration::CURRENT.l2_peak_uncertainty_milli
}

fn certified_operator_consensus(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    let selected_class = predicted_state_id(
        crate::nanda_wave::phase_field::hash_text(&event.original),
        evaluation.action.operator.as_str(),
        &candidate.replacement,
    );
    verified_operator_consensus_witness(candidate, evaluation)
        && matches!(
            evaluation.signals.l4_hidden_disposition,
            L4HiddenDisposition::Resolved | L4HiddenDisposition::Witnessed
        )
        && evaluation.signals.l4_hidden_selected_class == selected_class
        && evaluation.signals.l4_hidden_certificate_valid
        && (evaluation.signals.l4_hidden_disposition == L4HiddenDisposition::Resolved
            || (evaluation.signals.l4_hidden_selected_witnessed
                && evaluation.signals.l4_hidden_probe
                    == crate::nanda_wave::l4_active_disambiguation::L4WitnessProbe::OperatorConsensus
                        .as_str()))
}

fn is_precise_lexical_operator(operator: crate::language_action::LanguageActionOperator) -> bool {
    use crate::language_action::LanguageActionOperator;
    matches!(
        operator,
        LanguageActionOperator::FixTransposition
            | LanguageActionOperator::ReplaceLetter
            | LanguageActionOperator::RemoveExtraLetter
            | LanguageActionOperator::RestoreMissingLetter
            | LanguageActionOperator::NormalizeCase
    )
}

fn phase_lattice_strength(signals: &CandidateDecisionSignals) -> Option<f32> {
    interference::normalized_phase_lattice_strength(phase_readout_from_signals(signals))
}

fn phase_readout_from_signals(
    signals: &CandidateDecisionSignals,
) -> crate::nanda_wave::PhaseReadout {
    crate::nanda_wave::PhaseReadout {
        package_loaded: signals.l2_transition_phase_package_loaded,
        operator_present: signals.l2_transition_phase_operator_present,
        operator_promoted: signals.l2_transition_phase_operator_promoted,
        margin_micro: signals.l2_transition_phase_margin_micro,
        threshold_micro: signals.l2_transition_phase_threshold_micro,
        lexical_margin_micro: signals.l2_lexical_phase_margin_micro,
        lexical_threshold_micro: signals.l2_lexical_phase_threshold_micro,
        lexical_competition_ready: signals.l2_lexical_phase_competition_ready,
        positive_centers: signals.l2_transition_phase_positive_centers,
        anti_centers: signals.l2_transition_phase_anti_centers,
        covered_surfaces: signals.l2_transition_phase_surfaces,
        verdict: signals.l2_transition_phase_verdict,
        ..crate::nanda_wave::PhaseReadout::default()
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
            if candidate.has_origin(crate::candidate_contract::CandidateOrigin::DeterministicTypo) {
                0.08
            } else if candidate.has_origin(crate::candidate_contract::CandidateOrigin::L2Surface)
                && candidate.error_class == TypingErrorClass::SparseInternalMultiOmission
            {
                0.20
            } else if candidate.has_origin(crate::candidate_contract::CandidateOrigin::L2Surface)
                && action.operator
                    == crate::language_action::LanguageActionOperator::RestoreMissingLetter
            {
                0.12
            } else if candidate.has_origin(crate::candidate_contract::CandidateOrigin::L2Surface)
                && is_precise_lexical_operator(action.operator)
            {
                0.02
            } else if candidate.has_origin(crate::candidate_contract::CandidateOrigin::L2Surface) {
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
    let composition_cost =
        if candidate.origin == crate::candidate_contract::CandidateOrigin::LayoutThenTypo {
            0.12
        } else {
            0.0
        };
    operator_bonus - composition_cost
}

fn micro_to_milli(value: i64) -> i16 {
    (value / 1000).clamp(i16::MIN as i64, i16::MAX as i64) as i16
}
