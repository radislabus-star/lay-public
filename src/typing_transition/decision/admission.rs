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
    _policy: TransitionDecisionPolicy,
) -> bool {
    let candidate = &candidates[candidate_index];
    let evaluation = &evaluations[candidate_index];
    let bayes = &evaluation.bayes;
    let signals = &evaluation.signals;
    let source_role = candidate.origin.source_role();
    let exact_positive_transition = evaluation.transition.l4_signed_signal.exact_positive();
    let operator_consensus_authority = certified_operator_consensus(event, candidate, evaluation);
    let verified_l2_center_repair =
        verified_current_token_l2_center_repair(event, candidate_index, candidates, evaluations);
    if let Some(reason) = hard_structural_veto::hidden_state_rejection(signals) {
        if (hidden_rejection_deferred_to_verified_boundary(reason)
            && (verified_current_token_boundary_merge_split(event, candidate, evaluation)
                || hard_structural_veto::verified_tail_boundary_shift(
                    event, candidate, evaluation,
                )))
            || (hidden_rejection_deferred_to_verified_l2_repair(reason)
                && verified_l2_center_repair)
            || (hidden_rejection_deferred_to_verified_deterministic_repair(reason)
                && verified_current_token_deterministic_typo_repair(event, candidate, evaluation))
        {
            // L4 ambiguity protects lexical choice operators from guessing. A
            // verifier-proven boundary edit or strongly separated
            // dirty-surface -> lexical-center repair is already a typed edit
            // certificate; only exact negative memory may veto it later.
        } else {
            debug_decision_reject(candidate, reason, bayes.posterior, bayes.risk);
            return false;
        }
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
    let learned_short_boundary_authority = signals.l3_phrase_milli >= CURRENT.l3_strong_milli
        || signals.l4_signed_milli >= CURRENT.l4_strong_milli
        || exact_positive_transition;
    let context_state_support = super::calibration::known_word_context_state_support(
        bayes.context_prior,
        signals.l3_phrase_milli,
        signals.l4_signed_milli,
    );
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
    if known_word_transposition_requires_relation_proof(event, candidate)
        && !signals.l3_pairwise_certified
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "known_word_transposition_needs_pairwise_proof",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if short_transposition_requires_state_proof(event, candidate)
        && !context_state_support
        && !signals.l3_pairwise_certified
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "short_transposition_needs_state_proof",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if short_function_word_repair_requires_state_proof(event, candidate)
        && !signals.l3_pairwise_certified
        && signals.l3_phrase_milli < CURRENT.l3_strong_milli
        && signals.l4_signed_milli < CURRENT.l4_strong_milli
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "short_function_word_needs_state_proof",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if ambiguous_l2_surface_repair_requires_context(source_role, signals)
        && !context_state_support
        && !signals.l3_pairwise_certified
        && signals.l3_phrase_milli < CURRENT.l3_strong_milli
        && signals.l4_signed_milli < CURRENT.l4_strong_milli
        && !l2_transition_phase_supports_candidate(signals)
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "ambiguous_l2_surface_needs_context",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if source_role == CorrectionSourceRole::Layout
        && !signals.l3_pairwise_certified
        && signals.l3_phrase_milli < CURRENT.l3_strong_milli
        && !exact_positive_transition
        && close_unresolved_competitor_exists(event, candidate_index, candidates, evaluations)
    {
        debug_decision_reject(
            candidate,
            "ambiguous_layout_projection_needs_context",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if known_form_drift_requires_state_proof(event, candidate)
        && !super::calibration::known_word_drift_has_authority(
            context_state_support,
            exact_positive_transition,
        )
        && !signals.l3_pairwise_certified
    {
        debug_decision_reject(
            candidate,
            "known_form_drift_needs_state_proof",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if preposition_governed_inflection_deletion_requires_context(event, candidate)
        && !signals.l3_pairwise_certified
        && signals.l3_phrase_milli < CURRENT.l3_strong_milli
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "preposition_inflection_deletion_needs_context",
            bayes.posterior,
            bayes.risk,
        );
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
    let hidden_state_support = hidden_state_confirms_candidate(signals);
    let contextual_transition_support = bayes.context_prior >= CURRENT.learned_prior_floor
        || signals.l3_phrase_milli >= CURRENT.l3_strong_milli
        || signals.l4_signed_milli >= CURRENT.l4_strong_milli
        || exact_positive_transition
        || hidden_state_support
        || strong_l2_peak_support
        || verified_l2_center_repair;
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
    let strong_learned_support = external_learned_support
        || strong_l2_peak_support
        || high_precision_boundary_shift
        || verified_l2_center_repair;
    // A phase package may order candidates but may never manufacture apply
    // authority. Only independently verified state evidence reaches here.
    let verified_layout_projection = verified_layout_transition(&evaluation.transition);
    let strong_transition_support =
        context_state_support || verified_l2_center_repair || verified_layout_projection;
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
    if bayes.posterior < CURRENT.transition_posterior_floor
        && !strong_learned_support
        && !strong_transition_support
    {
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
    let allowed = !super::apply_policy::unresolved_competitor_blocks(
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

pub(super) fn suggest_boundary_allows_authority_evaluation(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    candidate.gate.action == CandidateGateAction::SuggestOnly
        && candidate.origin.source_role() == CorrectionSourceRole::Boundary
        && hard_structural_veto::verified_tail_boundary_shift(event, candidate, evaluation)
}

/// A swap between two already valid lexical states is ambiguous by surface
/// alone. L2 may propose it, but only a directional L3 pair certificate or an
/// exact accepted L4 transition can authorize changing user text.
fn known_word_transposition_requires_relation_proof(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> bool {
    if candidate.error_class != TypingErrorClass::AdjacentTransposition {
        return false;
    }
    stable_current_word_center(&event.original)
        && stable_current_word_center(&candidate.replacement)
}

fn short_transposition_requires_state_proof(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> bool {
    if candidate.error_class != TypingErrorClass::AdjacentTransposition {
        return false;
    }
    let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
        return false;
    };
    if !cyrillic_letters_only(&original) || !cyrillic_letters_only(&replacement) {
        return false;
    }
    original.chars().count().max(replacement.chars().count()) <= 3
        && damerau_levenshtein(&original, &replacement) <= 1
}

fn short_function_word_repair_requires_state_proof(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> bool {
    if !matches!(
        candidate.origin.source_role(),
        CorrectionSourceRole::DeterministicTypo | CorrectionSourceRole::L2Surface
    ) || !matches!(
        candidate.error_class,
        TypingErrorClass::RepeatedLetter
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::CompositeTypo
    ) {
        return false;
    }
    let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
        return false;
    };
    cyrillic_letters_only(&original)
        && cyrillic_letters_only(&replacement)
        && replacement.chars().count() <= 3
        && original != replacement
        && damerau_levenshtein(&original, &replacement) <= 2
        && crate::phrase_lexicon::is_short_russian_function_word(&replacement)
}

fn ambiguous_l2_surface_repair_requires_context(
    source_role: CorrectionSourceRole,
    signals: &CandidateDecisionSignals,
) -> bool {
    source_role == CorrectionSourceRole::L2Surface
        && signals.l4_hidden_disposition == L4HiddenDisposition::Ambiguous
        && signals.l4_hidden_certificate_valid
        && signals.l4_hidden_selected_class == 0
        && signals.l4_hidden_semantic_classes >= 4
        && signals.l4_hidden_unresolved_classes > 0
}

fn l2_transition_phase_supports_candidate(signals: &CandidateDecisionSignals) -> bool {
    signals.l2_transition_phase_operator_promoted
        && signals.l2_transition_phase_verdict == crate::nanda_wave::PhaseVerdict::Support
        && signals.l2_transition_phase_milli > 0
        && signals.l2_transition_phase_milli >= signals.l2_transition_phase_threshold_milli
}

fn known_form_drift_requires_state_proof(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> bool {
    if !matches!(
        candidate.error_class,
        TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
    ) {
        return false;
    }
    let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
        return false;
    };
    if original == replacement
        || !cyrillic_letters_only(&original)
        || !cyrillic_letters_only(&replacement)
        || damerau_levenshtein(&original, &replacement) > 2
    {
        return false;
    }
    known_observed_lexical_state(&original) && known_lexical_state_or_form(&replacement)
}

fn preposition_governed_inflection_deletion_requires_context(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> bool {
    if candidate.origin.source_role() != CorrectionSourceRole::L2Surface
        || candidate.error_class != TypingErrorClass::ExtraLetter
    {
        return false;
    }
    let words =
        crate::typing_transition::proposal_admission::normalized_correction_words(&event.original);
    let Some(previous) = words
        .len()
        .checked_sub(2)
        .and_then(|index| words.get(index))
        .map(|word| word.to_lowercase())
    else {
        return false;
    };
    if !crate::lexicon::is_ru_short_preposition(&previous)
        && !matches!(previous.as_str(), "в" | "к" | "с" | "о")
    {
        return false;
    }
    let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
        return false;
    };
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    original_chars.len() >= 4
        && original_chars.len() == replacement_chars.len() + 1
        && original_chars[..replacement_chars.len()] == replacement_chars
        && original_chars
            .last()
            .is_some_and(|ch| crate::russian_chars::is_russian_vowel(*ch))
}

fn current_and_replacement_words(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> Option<(String, String)> {
    Some((
        event.current_word.to_lowercase(),
        last_replacement_word(&candidate.replacement)?.to_lowercase(),
    ))
}

fn known_lexical_state_or_form(word: &str) -> bool {
    let field = crate::hot_field::HotFieldSnapshot::current();
    field.form_readout(word).has_structural_center()
        || crate::lexicon::is_common_ru_word(word)
        || crate::lexicon::is_ru_live_protected_word(word)
        || crate::lexicon::is_user_protected_word(word)
        || known_russian_form(word)
}

fn known_observed_lexical_state(word: &str) -> bool {
    let field = crate::hot_field::HotFieldSnapshot::current();
    field.surface_phase_readout(word).exact_center
        || crate::lexicon::is_common_ru_word(word)
        || crate::lexicon::is_ru_live_protected_word(word)
        || crate::lexicon::is_user_protected_word(word)
        || crate::russian_lexicon::has_clean_russian_surface_certificate(word)
        || crate::typing_transition::state::word_has_common_usage_authority(word)
}

fn known_russian_form(word: &str) -> bool {
    crate::russian_lexicon::is_known_russian_word_or_form(word)
        || crate::russian_lexicon::is_known_russian_adverb_o_form(word)
        || crate::russian_lexicon::is_known_russian_ka_oblique_form(word)
}

fn cyrillic_letters_only(word: &str) -> bool {
    !word.is_empty() && word.chars().all(is_cyrillic_letter)
}

fn stable_current_word_center(text: &str) -> bool {
    let Some(word) = crate::word_reader::last_text_word(text) else {
        return false;
    };
    crate::hot_field::HotFieldSnapshot::current()
        .word_readout(&word)
        .has_phase_authority()
}

#[cfg(test)]
mod known_word_transposition_tests {
    use super::*;
    use crate::candidate_contract::CandidateOrigin;
    use crate::correction_core::{
        CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource,
    };

    fn event(text: &str) -> TypingErrorEvent {
        TypingErrorEvent {
            original: text.to_string(),
            core: text.trim().to_string(),
            current_word: text
                .split_whitespace()
                .last()
                .unwrap_or_default()
                .to_string(),
            input_class: TypingErrorClass::AdjacentTransposition,
        }
    }

    fn candidate(replacement: &str) -> UnifiedCorrectionCandidate {
        UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "CanonicalL2FieldSurface",
            TypingErrorClass::AdjacentTransposition,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        )
    }

    #[test]
    fn ambiguous_known_to_known_swap_requires_relation_proof() {
        assert!(known_word_transposition_requires_relation_proof(
            &event("он "),
            &candidate("но "),
        ));
    }

    #[test]
    fn unknown_to_known_transposition_remains_an_l2_repair() {
        assert!(!known_word_transposition_requires_relation_proof(
            &event("ландо "),
            &candidate("ладно "),
        ));
    }
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
    let candidate_rank = evaluations[candidate_index].signals.rank_score;

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
            && candidate_rank
                <= evaluations[other_index].signals.rank_score + CURRENT.structural_rank_proximity
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

fn verified_current_token_boundary_merge_split(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    candidate.origin.source_role() == CorrectionSourceRole::Boundary
        && matches!(
            candidate.error_class,
            TypingErrorClass::GluedWords | TypingErrorClass::SplitWord
        )
        && evaluation.action.verifier_passed
        && evaluation.action.edit_operator == verifier::EditTransitionOperator::BoundaryMergeSplit
        && crate::text_metrics::current_token_boundary_split_or_repair(
            &event.original,
            &candidate.replacement,
        )
}

fn hidden_rejection_deferred_to_verified_boundary(reason: &str) -> bool {
    matches!(reason, "ambiguous" | "unobserved")
}

fn hidden_rejection_deferred_to_verified_l2_repair(reason: &str) -> bool {
    reason == "unobserved"
}

fn hidden_rejection_deferred_to_verified_deterministic_repair(reason: &str) -> bool {
    reason == "unobserved"
}

fn verified_current_token_l2_center_repair(
    event: &TypingErrorEvent,
    candidate_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> bool {
    let candidate = &candidates[candidate_index];
    let evaluation = &evaluations[candidate_index];
    if candidate.origin.source_role() != CorrectionSourceRole::L2Surface
        || !matches!(
            candidate.error_class,
            TypingErrorClass::MissingLetter
                | TypingErrorClass::SparseInternalMultiOmission
                | TypingErrorClass::LetterSubstitution
                | TypingErrorClass::ExtraLetter
                | TypingErrorClass::RepeatedLetter
                | TypingErrorClass::AdjacentTransposition
                | TypingErrorClass::CompositeTypo
        )
        || !evaluation.action.verifier_passed
        || evaluation.action.left_context_changed
        || evaluation.action.changed_tokens != 1
        || evaluation.action.edit_operator != verifier::EditTransitionOperator::ReplaceCurrentWord
    {
        return false;
    }
    let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
        return false;
    };
    if !cyrillic_letters_only(&original)
        || !cyrillic_letters_only(&replacement)
        || original == replacement
        || stable_observed_lexical_word_blocks_repair(&original)
        || !known_lexical_state_or_form(&replacement)
    {
        return false;
    }
    let distance = damerau_levenshtein(&original, &replacement);
    let typed_geometry = distance <= 1
        || crate::text_metrics::sparse_internal_omission_count(&original, &replacement).is_some()
        || (candidate.error_class == TypingErrorClass::CompositeTypo && distance <= 3);
    typed_geometry
        && (strong_l2_wave_peak_support(&evaluation.signals)
            || phase_center_separates_candidate(event, candidate_index, candidates, evaluations))
}

fn verified_current_token_deterministic_typo_repair(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    if candidate.origin.source_role() != CorrectionSourceRole::DeterministicTypo
        || !matches!(
            candidate.error_class,
            TypingErrorClass::MissingLetter
                | TypingErrorClass::LetterSubstitution
                | TypingErrorClass::ExtraLetter
                | TypingErrorClass::RepeatedLetter
                | TypingErrorClass::AdjacentTransposition
                | TypingErrorClass::CompositeTypo
        )
        || !evaluation.action.verifier_passed
        || evaluation.action.left_context_changed
        || evaluation.action.changed_tokens != 1
        || evaluation.action.edit_operator != verifier::EditTransitionOperator::ReplaceCurrentWord
    {
        return false;
    }
    let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
        return false;
    };
    if !cyrillic_letters_only(&original)
        || !cyrillic_letters_only(&replacement)
        || original == replacement
        || stable_observed_lexical_word_blocks_repair(&original)
        || !known_lexical_state_or_form(&replacement)
    {
        return false;
    }
    let distance = damerau_levenshtein(&original, &replacement);
    distance <= 1
        || crate::text_metrics::sparse_internal_omission_count(&original, &replacement).is_some()
        || (candidate.error_class == TypingErrorClass::CompositeTypo && distance <= 2)
}

fn stable_observed_lexical_word_blocks_repair(word: &str) -> bool {
    known_observed_lexical_state(word)
}

fn hidden_state_confirms_candidate(signals: &CandidateDecisionSignals) -> bool {
    matches!(
        signals.l4_hidden_disposition,
        L4HiddenDisposition::Resolved | L4HiddenDisposition::Witnessed
    ) && signals.l4_hidden_certificate_valid
        && signals.l4_hidden_selected_class != 0
}

pub(super) fn admit_evaluated_hidden_transition(
    _candidate_count: usize,
    _source_role: CorrectionSourceRole,
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

    if hidden_short_transposition_requires_state_proof(transition)
        && !super::calibration::known_word_drift_has_authority(
            context_state_support,
            exact_state_support,
        )
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "short_transposition_needs_state_proof",
        };
    }

    if transition
        .state_before
        .known_word_drift_to(&transition.state_after_predicted)
        && !super::calibration::known_word_drift_has_authority(
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
        && generic_l4_negative_can_veto(transition, operator_consensus_witness)
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

fn generic_l4_negative_can_veto(
    transition: &TypingTransition,
    operator_consensus_witness: bool,
) -> bool {
    if transition.l4_signed_signal.state_specific {
        return true;
    }
    !operator_consensus_witness && !verified_layout_transition(transition)
}

fn verified_layout_transition(transition: &TypingTransition) -> bool {
    transition.evidence.verifier_passed
        && !transition.evidence.left_context_changed
        && !transition.l1_signal.word_count_changed
        && matches!(
            transition.evidence.origin,
            crate::candidate_contract::CandidateOrigin::Layout
                | crate::candidate_contract::CandidateOrigin::LayoutThenTypo
        )
        && matches!(
            transition.evidence.edit_proof,
            crate::language_action::LanguageActionProof::Layout
        )
        && matches!(
            transition.evidence.edit_operator,
            crate::text_edit::TransitionOperator::LayoutProjection
                | crate::text_edit::TransitionOperator::ReplaceCurrentWord
        )
}

fn hidden_short_transposition_requires_state_proof(transition: &TypingTransition) -> bool {
    transition.evidence.error_class == TypingErrorClass::AdjacentTransposition
        && transition
            .state_before
            .current_word_changed(&transition.state_after_predicted)
        && transition.state_before.script == transition.state_after_predicted.script
        && cyrillic_letters_only(&transition.state_before.current_word)
        && cyrillic_letters_only(&transition.state_after_predicted.current_word)
        && transition.state_before.current_word.chars().count().max(
            transition
                .state_after_predicted
                .current_word
                .chars()
                .count(),
        ) <= 3
        && damerau_levenshtein(
            &transition.state_before.current_word,
            &transition.state_after_predicted.current_word,
        ) <= 1
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
    if selected_is_complete_proven_transition
        || verified_current_token_l2_center_repair(event, selected_index, candidates, evaluations)
    {
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
    if selected_signal.l2_transition_phase_operator_promoted
        && selected_signal.l2_transition_phase_verdict == crate::nanda_wave::PhaseVerdict::Repel
        && selected_signal.l2_transition_phase_milli < 0
    {
        return false;
    }
    let strongest_lexical_competitor = candidates
        .iter()
        .enumerate()
        .filter(|(candidate_index, _)| *candidate_index != selected_index)
        .filter(|(_, candidate)| candidate.gate.action != CandidateGateAction::Veto)
        .filter(|(_, candidate)| {
            matches!(
                candidate.origin.source_role(),
                CorrectionSourceRole::Layout
                    | CorrectionSourceRole::DeterministicTypo
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
                CorrectionSourceRole::Layout
                    | CorrectionSourceRole::DeterministicTypo
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
