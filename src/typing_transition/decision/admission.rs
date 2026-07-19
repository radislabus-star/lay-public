use super::calibration::CURRENT;
use super::hard_structural_veto;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransitionAdmission {
    pub(crate) allow_apply: bool,
    pub(crate) reason: &'static str,
}

pub(super) fn candidate_has_apply_authority(
    event: &TypingErrorEvent,
    candidate_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
    policy: TransitionDecisionPolicy,
) -> bool {
    let candidate = &candidates[candidate_index];
    let evaluation = &evaluations[candidate_index];
    let bayes = &evaluation.bayes;
    let signals = &evaluation.signals;
    let source_role = candidate.origin.source_role();
    let exact_positive_transition = evaluation.transition.l4_signed_signal.exact_positive();
    let operator_consensus_authority = certified_operator_consensus(event, candidate, evaluation);
    if let Some(reason) = hard_structural_veto::hidden_state_rejection(signals) {
        debug_decision_reject(candidate, reason, bayes.posterior, bayes.risk);
        return false;
    }
    if source_role == CorrectionSourceRole::L3Context
        && signals.l3_phrase_milli < CURRENT.l3_strong_milli
        && signals.l4_signed_milli < CURRENT.l4_strong_milli
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "l3_context_evidence_absent",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if let Some(reason) = phase_policy_rejection(
        policy,
        source_role,
        signals.l2_transition_phase_package_loaded,
        signals.l2_transition_phase_operator_present,
        signals.l2_transition_phase_operator_promoted,
        signals.l2_transition_phase_verdict,
    ) {
        debug_decision_reject(candidate, reason, bayes.posterior, bayes.risk);
        return false;
    }
    let learned_short_boundary_authority = signals.l3_phrase_milli >= CURRENT.l3_strong_milli
        || signals.l4_signed_milli >= CURRENT.l4_strong_milli
        || exact_positive_transition;
    let high_precision_boundary_shift =
        hard_structural_veto::high_precision_boundary_shift(event, candidate, evaluation);
    if let Some(reason) = hard_structural_veto::boundary_shift_rejection(
        event,
        candidate,
        evaluation,
        learned_short_boundary_authority,
    ) {
        debug_decision_reject(candidate, reason, bayes.posterior, bayes.risk);
        return false;
    }
    if let Some(reason) = hard_structural_veto::verifier_rejection(evaluation) {
        debug_decision_reject(candidate, reason, bayes.posterior, bayes.risk);
        return false;
    }
    let verified_mass_preserving_l2_transition =
        is_verified_mass_preserving_l2_transition(source_role, candidate, evaluation);
    let self_referential_surface_drift = source_role == CorrectionSourceRole::L2Surface
        && short_same_length_surface_drift(&event.current_word, &candidate.replacement)
        && !verified_mass_preserving_l2_transition;
    let strong_l2_peak_support =
        strong_l2_wave_peak_support(signals) && !self_referential_surface_drift;
    let external_learned_support = bayes.usage_prior >= CURRENT.learned_prior_floor
        || bayes.context_prior >= CURRENT.learned_prior_floor
        || signals.l3_phrase_milli >= CURRENT.l3_strong_milli
        || signals.l4_signed_milli >= CURRENT.l4_strong_milli;
    let contextual_transition_support = bayes.context_prior >= CURRENT.learned_prior_floor
        || signals.l3_phrase_milli >= CURRENT.l3_strong_milli
        || signals.l4_signed_milli >= CURRENT.l4_strong_milli
        || exact_positive_transition
        || strong_l2_peak_support;
    if candidate.origin == crate::candidate_contract::CandidateOrigin::LayoutThenTypo
        && original_tail_has_same_script_context(event)
        && !contextual_transition_support
    {
        debug_decision_reject(
            candidate,
            "composed_layout_needs_context_proof",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    let strong_learned_support =
        external_learned_support || strong_l2_peak_support || high_precision_boundary_shift;
    let context_state_support = known_word_context_state_support(
        bayes.context_prior,
        signals.l3_phrase_milli,
        signals.l4_signed_milli,
    );
    let strong_transition_support = context_state_support
        || (policy.l2_phase_apply
            && signals.l2_transition_phase_operator_promoted
            && signals.l2_transition_phase_verdict == crate::nanda_wave::PhaseVerdict::Support
            && signals.l2_transition_phase_milli >= signals.l2_transition_phase_threshold_milli
            && signals.l2_transition_phase_surfaces >= 3);
    let admission = admit_evaluated_hidden_transition(
        candidates.len(),
        source_role,
        context_state_support,
        operator_consensus_authority,
        &evaluation.transition,
    );
    if !admission.allow_apply {
        debug_decision_reject(candidate, admission.reason, bayes.posterior, bayes.risk);
        return false;
    }
    if !exact_positive_transition
        && learned_candidate_shadowed_by_deterministic_owner(
            event,
            candidate_index,
            candidates,
            evaluations,
            source_role,
        )
    {
        debug_decision_reject(
            candidate,
            "deterministic_owner_gravity",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if bayes.risk >= CURRENT.high_risk_floor && !strong_transition_support {
        debug_decision_reject(candidate, "high_risk", bayes.posterior, bayes.risk);
        return false;
    }
    if bayes.posterior < CURRENT.transition_posterior_floor && !strong_learned_support {
        debug_decision_reject(candidate, "low_posterior", bayes.posterior, bayes.risk);
        return false;
    }
    if self_referential_surface_drift && !external_learned_support {
        debug_decision_reject(
            candidate,
            "short_same_length_surface_drift",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if source_role == CorrectionSourceRole::L2Surface
        && candidate.error_class == TypingErrorClass::CompositeTypo
        && !external_learned_support
        && !exact_positive_transition
        && lexical_transition_distance(event, candidate) >= 2
        && !phase_center_separates_candidate(event, candidate_index, candidates, evaluations)
        && competing_lexical_margin(event, candidate_index, candidates, evaluations)
            < CURRENT.composite_margin_floor
    {
        debug_decision_reject(
            candidate,
            "l2_composite_margin_low",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if !strong_transition_support
        && !phase_center_separates_candidate(event, candidate_index, candidates, evaluations)
        && close_unresolved_competitor_exists(event, candidate_index, candidates, evaluations)
    {
        debug_decision_reject(
            candidate,
            "ambiguous_transition_margin",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    let allowed = !unresolved_competitor_blocks(
        exact_positive_transition || operator_consensus_authority,
        stronger_unresolved_candidate_exists(event, candidate_index, candidates, evaluations),
    );
    if !allowed {
        debug_decision_reject(
            candidate,
            "stronger_unresolved_transition",
            bayes.posterior,
            bayes.risk,
        );
    }
    allowed
}

fn close_unresolved_competitor_exists(
    event: &TypingErrorEvent,
    selected_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> bool {
    let selected = &evaluations[selected_index];
    let selected_origin = candidates[selected_index].origin;
    let selected_distance = lexical_transition_distance(event, &candidates[selected_index]);
    let selected_span =
        changed_token_span(&event.original, &candidates[selected_index].replacement);
    candidates.iter().enumerate().any(|(index, candidate)| {
        if index == selected_index
            || candidate.origin != selected_origin
            || candidate.gate.action != CandidateGateAction::Eligible
            || lexical_transition_distance(event, candidate) > selected_distance
        {
            return false;
        }
        let competitor = &evaluations[index];
        competitor.action.verifier_passed
            && changed_spans_overlap(
                selected_span,
                changed_token_span(&event.original, &candidate.replacement),
            )
            && (selected.signals.rank_score - competitor.signals.rank_score).abs()
                < CURRENT.structural_rank_proximity
    })
}

fn original_tail_has_same_script_context(event: &TypingErrorEvent) -> bool {
    let words = crate::correction_core::normalized_correction_words(&event.original);
    let Some((current, left)) = words.split_last() else {
        return false;
    };
    if left.is_empty() {
        return false;
    }
    let current_is_ru = current.chars().all(is_cyrillic_letter);
    let current_is_en = current.chars().all(|ch| ch.is_ascii_alphabetic());
    left.iter().rev().take(3).any(|word| {
        (current_is_ru && word.chars().all(is_cyrillic_letter))
            || (current_is_en && word.chars().all(|ch| ch.is_ascii_alphabetic()))
    })
}

fn phase_managed_source(source_role: CorrectionSourceRole) -> bool {
    matches!(
        source_role,
        CorrectionSourceRole::DeterministicTypo
            | CorrectionSourceRole::L2Surface
            | CorrectionSourceRole::L3Context
    )
}

pub(super) fn phase_policy_rejection(
    policy: TransitionDecisionPolicy,
    source_role: CorrectionSourceRole,
    package_loaded: bool,
    operator_present: bool,
    operator_promoted: bool,
    verdict: crate::nanda_wave::PhaseVerdict,
) -> Option<&'static str> {
    if !policy.l2_phase_apply || !phase_managed_source(source_role) {
        return None;
    }
    if !package_loaded {
        return Some("l2_transition_phase_package_missing");
    }
    if !operator_present {
        return Some("l2_transition_phase_operator_missing");
    }
    if !operator_promoted {
        return Some("l2_transition_phase_shadow_only");
    }
    match verdict {
        crate::nanda_wave::PhaseVerdict::Repel => Some("l2_transition_phase_repel"),
        crate::nanda_wave::PhaseVerdict::Unknown => Some("l2_transition_phase_unknown"),
        crate::nanda_wave::PhaseVerdict::Support => None,
    }
}

fn learned_candidate_shadowed_by_deterministic_owner(
    event: &TypingErrorEvent,
    candidate_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
    source_role: CorrectionSourceRole,
) -> bool {
    let candidate = &candidates[candidate_index];
    if candidate.source != CorrectionDecisionSource::Nanda
        || !matches!(
            source_role,
            CorrectionSourceRole::L2Surface | CorrectionSourceRole::L3Context
        )
    {
        return false;
    }
    let Some(candidate_word) = last_replacement_word(&candidate.replacement) else {
        return false;
    };
    let original_word = event.current_word.to_lowercase();
    let candidate_distance = damerau_levenshtein(&original_word, &candidate_word.to_lowercase());

    candidates.iter().enumerate().any(|(other_index, other)| {
        if other.source != CorrectionDecisionSource::Deterministic
            || other.gate.action != CandidateGateAction::Eligible
        {
            return false;
        }
        let other_role = other.origin.source_role();
        if !matches!(
            other_role,
            CorrectionSourceRole::DeterministicTypo
                | CorrectionSourceRole::Layout
                | CorrectionSourceRole::Boundary
        ) {
            return false;
        }
        let transition = &evaluations[other_index].transition;
        if !transition.evidence.verifier_passed
            || transition.evidence.left_context_changed
            || transition.l4_signed_signal.negative
        {
            return false;
        }
        let Some(other_word) = last_replacement_word(&other.replacement) else {
            return false;
        };
        let other_distance = damerau_levenshtein(&original_word, &other_word.to_lowercase());
        other_distance <= candidate_distance
    })
}

fn is_verified_mass_preserving_l2_transition(
    source_role: CorrectionSourceRole,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    source_role == CorrectionSourceRole::L2Surface
        && verified_mass_preserving_l2_transition(candidate, evaluation)
}

pub(super) fn admit_evaluated_hidden_transition(
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    context_state_support: bool,
    operator_consensus_witness: bool,
    transition: &TypingTransition,
) -> TransitionAdmission {
    let exact_state_support = transition.l4_signed_signal.exact_positive();
    if transition
        .state_before
        .candidate_imported_left_context(&transition.state_after_predicted)
        && !transition.evidence.verifier_passed
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_context_import",
        };
    }

    if transition
        .state_before
        .context_changed(&transition.state_after_predicted)
        && !transition.evidence.verifier_passed
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_context_unverified",
        };
    }

    if transition
        .state_before
        .known_word_drift_to(&transition.state_after_predicted)
        && !known_word_drift_has_authority(
            source_role,
            candidate_count,
            context_state_support,
            exact_state_support,
        )
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_known_word_drift_needs_state_proof",
        };
    }

    // Exact rejected experience is authoritative. A generic anti-state remains
    // ranking pressure, but cannot veto an independently verified operator for
    // this candidate.
    if transition.l4_signed_signal.negative
        && (transition.l4_signed_signal.state_specific || !operator_consensus_witness)
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_l4_negative_transition_memory",
        };
    }

    TransitionAdmission {
        allow_apply: true,
        reason: "latent_transition_admitted",
    }
}

fn known_word_context_state_support(
    context_prior: f32,
    l3_phrase_milli: i16,
    l4_signed_milli: i16,
) -> bool {
    context_prior >= CURRENT.learned_prior_floor
        || l3_phrase_milli >= CURRENT.l3_strong_milli
        || l4_signed_milli >= CURRENT.l4_strong_milli
}

pub(super) fn known_word_drift_has_authority(
    _source_role: CorrectionSourceRole,
    _candidate_count: usize,
    strong_state_support: bool,
    exact_state_support: bool,
) -> bool {
    exact_state_support || strong_state_support
}

fn short_same_length_surface_drift(original_word: &str, replacement: &str) -> bool {
    let Some(replacement_word) = last_replacement_word(replacement) else {
        return false;
    };
    let original = original_word.to_lowercase();
    let replacement = replacement_word.to_lowercase();
    let original_len = original.chars().count();
    original_len <= 6
        && original_len == replacement.chars().count()
        && original.chars().all(is_cyrillic_letter)
        && replacement.chars().all(is_cyrillic_letter)
        && damerau_levenshtein(&original, &replacement) <= 2
}

fn last_replacement_word(text: &str) -> Option<String> {
    text.split_whitespace().rev().find_map(|token| {
        let (_, word, _) = split_word_punctuation(token);
        (!word.is_empty()).then(|| word.to_string())
    })
}

fn debug_decision_reject(
    candidate: &UnifiedCorrectionCandidate,
    reason: &'static str,
    posterior: f32,
    risk: f32,
) {
    if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
        eprintln!(
            "decision-core-reject reason={reason} source_id={} class={} replacement={:?} posterior={:.3} risk={:.3}",
            candidate.source_id,
            candidate.error_class.as_str(),
            candidate.replacement,
            posterior,
            risk
        );
    }
}

fn stronger_unresolved_candidate_exists(
    event: &TypingErrorEvent,
    selected_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> bool {
    let selected = &candidates[selected_index];
    let selected_evaluation = &evaluations[selected_index];
    let selected_action = selected_evaluation.action;
    let selected_is_complete_proven_transition = selected_action.verifier_passed
        && matches!(
            selected_action.edit_operator,
            verifier::EditTransitionOperator::BoundaryShift
                | verifier::EditTransitionOperator::BoundaryMergeSplit
                | verifier::EditTransitionOperator::LayoutProjection
        );
    if selected_is_complete_proven_transition {
        return false;
    }
    let selected_bayes = &selected_evaluation.bayes;
    let selected_signals = &selected_evaluation.signals;
    let selected_explanation = selected_evaluation.explanation;
    let selected_span = changed_token_span(&event.original, &selected.replacement);
    candidates
        .iter()
        .enumerate()
        .any(|(candidate_index, candidate)| {
            if candidate_index == selected_index
                || candidate.gate.action == CandidateGateAction::Veto
            {
                return false;
            }
            if !changed_spans_overlap(
                selected_span,
                changed_token_span(&event.original, &candidate.replacement),
            ) {
                return false;
            }
            let candidate_evaluation = &evaluations[candidate_index];
            let candidate_bayes = &candidate_evaluation.bayes;
            let candidate_signals = &candidate_evaluation.signals;
            let candidate_explanation = candidate_evaluation.explanation;
            let preservation_gain = candidate_explanation
                .preservation_milli
                .saturating_sub(selected_explanation.preservation_milli);
            let loss_reduction = selected_explanation
                .lost_mass_milli
                .saturating_sub(candidate_explanation.lost_mass_milli);
            let structurally_dominates = preservation_gain
                >= CURRENT.structural_preservation_gain_milli
                && loss_reduction >= CURRENT.structural_loss_reduction_milli
                && candidate_explanation.operator_fit_milli
                    >= selected_explanation.operator_fit_milli;
            candidate_bayes.risk <= selected_bayes.risk
                && (candidate_signals.rank_score > selected_signals.rank_score
                    || (structurally_dominates
                        && candidate_signals.rank_score + CURRENT.structural_rank_proximity
                            >= selected_signals.rank_score))
        })
}

fn changed_token_span(original: &str, replacement: &str) -> Option<(usize, usize)> {
    let original_words = crate::correction_core::normalized_correction_words(original);
    let replacement_words = crate::correction_core::normalized_correction_words(replacement);
    let width = original_words.len().max(replacement_words.len());
    let mut first = None;
    let mut last = 0usize;
    for index in 0..width {
        if original_words.get(index) != replacement_words.get(index) {
            first.get_or_insert(index);
            last = index;
        }
    }
    first.map(|first| (first, last))
}

fn changed_spans_overlap(left: Option<(usize, usize)>, right: Option<(usize, usize)>) -> bool {
    match (left, right) {
        (Some((left_start, left_end)), Some((right_start, right_end))) => {
            left_start <= right_end && right_start <= left_end
        }
        _ => false,
    }
}

fn lexical_transition_distance(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> usize {
    let original = last_replacement_word(&event.original).unwrap_or_default();
    let replacement = last_replacement_word(&candidate.replacement).unwrap_or_default();
    crate::text_metrics::damerau_levenshtein(&original, &replacement)
}

fn competing_lexical_margin(
    event: &TypingErrorEvent,
    selected_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> f32 {
    let selected = &candidates[selected_index];
    let selected_span = changed_token_span(&event.original, &selected.replacement);
    let selected_score = evaluations[selected_index].signals.rank_score;
    let runner_up = candidates
        .iter()
        .enumerate()
        .filter(|(candidate_index, _)| *candidate_index != selected_index)
        .filter(|(_, candidate)| candidate.gate.action != CandidateGateAction::Veto)
        .filter(|(_, candidate)| {
            matches!(
                candidate.origin.source_role(),
                CorrectionSourceRole::DeterministicTypo
                    | CorrectionSourceRole::L2Surface
                    | CorrectionSourceRole::L3Context
            )
        })
        .filter(|(_, candidate)| {
            changed_spans_overlap(
                selected_span,
                changed_token_span(&event.original, &candidate.replacement),
            )
        })
        .map(|(candidate_index, _)| evaluations[candidate_index].signals.rank_score)
        .max_by(f32::total_cmp);
    runner_up.map_or(f32::INFINITY, |score| selected_score - score)
}

fn phase_center_separates_candidate(
    event: &TypingErrorEvent,
    selected_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> bool {
    let selected = &candidates[selected_index];
    let selected_span = changed_token_span(&event.original, &selected.replacement);
    let selected_signal = &evaluations[selected_index].signals;
    let strongest_lexical_competitor = candidates
        .iter()
        .enumerate()
        .filter(|(candidate_index, _)| *candidate_index != selected_index)
        .filter(|(_, candidate)| candidate.gate.action != CandidateGateAction::Veto)
        .filter(|(_, candidate)| {
            matches!(
                candidate.origin.source_role(),
                CorrectionSourceRole::DeterministicTypo
                    | CorrectionSourceRole::L2Surface
                    | CorrectionSourceRole::L3Context
            )
        })
        .filter(|(_, candidate)| {
            changed_spans_overlap(
                selected_span,
                changed_token_span(&event.original, &candidate.replacement),
            )
        })
        .map(|(candidate_index, _)| {
            evaluations[candidate_index]
                .signals
                .l2_wave_peak_positive_milli
        })
        .max();
    if selected_signal.l2_wave_peak_milli >= CURRENT.l2_peak_milli
        && selected_signal.l2_wave_peak_uncertainty_milli <= CURRENT.l2_peak_uncertainty_milli
        && strongest_lexical_competitor.map_or(true, |competitor| {
            selected_signal
                .l2_wave_peak_positive_milli
                .saturating_sub(competitor)
                >= CURRENT.l2_competitor_gap_milli
        })
    {
        return true;
    }
    if !selected_signal.l2_transition_phase_operator_promoted
        || selected_signal.l2_transition_phase_verdict != crate::nanda_wave::PhaseVerdict::Support
        || selected_signal.l2_transition_phase_milli
            < selected_signal.l2_transition_phase_threshold_milli
    {
        return false;
    }
    let strongest_competitor = candidates
        .iter()
        .enumerate()
        .filter(|(candidate_index, _)| *candidate_index != selected_index)
        .filter(|(_, candidate)| candidate.gate.action != CandidateGateAction::Veto)
        .filter(|(_, candidate)| {
            matches!(
                candidate.origin.source_role(),
                CorrectionSourceRole::DeterministicTypo
                    | CorrectionSourceRole::L2Surface
                    | CorrectionSourceRole::L3Context
            )
        })
        .filter(|(_, candidate)| {
            changed_spans_overlap(
                selected_span,
                changed_token_span(&event.original, &candidate.replacement),
            )
        })
        .map(|(candidate_index, _)| {
            evaluations[candidate_index]
                .signals
                .l2_transition_phase_milli
        })
        .max();
    strongest_competitor.map_or(true, |competitor| {
        selected_signal
            .l2_transition_phase_milli
            .saturating_sub(competitor)
            >= CURRENT.phase_competitor_gap_milli
    })
}
