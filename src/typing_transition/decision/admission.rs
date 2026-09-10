use super::calibration::CURRENT;
use super::hard_structural_veto;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransitionAdmission {
    pub(crate) allow_apply: bool,
    pub(crate) reason: &'static str,
}

const LEXICAL_FIELD: u8 = 1 << 0;
const L3_DIRECTIONAL_PAIR: u8 = 1 << 1;
const L4_EXACT_STATE: u8 = 1 << 2;
const BOUNDARY_FIELD: u8 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CorrectionSafetyObservation {
    pub(super) candidate_tier: crate::typing_rule_graph::TypingRuleRequiredSafety,
    pub(super) required_domains: u8,
    pub(super) observed_domains: u8,
    pub(super) observed_domain_count: u8,
    pub(super) allow_apply: bool,
    pub(super) reason: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct CorrectionSafetyEvidence {
    pub(super) operator_consensus: bool,
    pub(super) verified_l2_center_repair: bool,
    pub(super) l3_directional_pair: bool,
    pub(super) l4_exact_state: bool,
    pub(super) high_precision_boundary: bool,
}

pub(super) fn observe_correction_safety(
    candidate: &UnifiedCorrectionCandidate,
    active_profile: crate::config::CorrectionSafety,
    evidence: CorrectionSafetyEvidence,
) -> CorrectionSafetyObservation {
    use crate::config::CorrectionSafety;
    use crate::typing_rule_graph::TypingRuleRequiredSafety;

    let mut candidate_tier = TypingRuleRequiredSafety::Experimental;
    let mut has_deterministic_alias = false;
    let mut has_nanda_alias = false;
    for alias in &candidate.evidence {
        candidate_tier = candidate_tier.min(alias_required_safety(alias));
        has_deterministic_alias |= alias.source == CorrectionDecisionSource::Deterministic;
        has_nanda_alias |= alias.source == CorrectionDecisionSource::Nanda;
    }
    let profile_rank: u8 = match active_profile {
        CorrectionSafety::Strict => 0,
        CorrectionSafety::Normal => 1,
        CorrectionSafety::Experimental => 2,
    };
    let tier_rank: u8 = match candidate_tier {
        TypingRuleRequiredSafety::Strict => 0,
        TypingRuleRequiredSafety::Normal => 1,
        TypingRuleRequiredSafety::Experimental => 2,
    };
    let required_domains = tier_rank.saturating_sub(profile_rank);

    let source_role = candidate.origin.source_role();
    let mut observed_domains = 0;
    if source_role != CorrectionSourceRole::Boundary
        && ((has_deterministic_alias && has_nanda_alias)
            || evidence.verified_l2_center_repair
            || evidence.operator_consensus)
    {
        observed_domains |= LEXICAL_FIELD;
    }
    if evidence.l3_directional_pair {
        observed_domains |= L3_DIRECTIONAL_PAIR;
    }
    if evidence.l4_exact_state {
        observed_domains |= L4_EXACT_STATE;
    }
    if source_role == CorrectionSourceRole::Boundary && evidence.high_precision_boundary {
        observed_domains |= BOUNDARY_FIELD;
    }
    let observed_domain_count = observed_domains.count_ones() as u8;
    let allow_apply = observed_domain_count >= required_domains;

    CorrectionSafetyObservation {
        candidate_tier,
        required_domains,
        observed_domains,
        observed_domain_count,
        allow_apply,
        reason: if allow_apply {
            "correction_safety_admitted"
        } else {
            "correction_safety_requires_more_evidence"
        },
    }
}

fn alias_required_safety(
    alias: &crate::correction_core::CandidateEvidence,
) -> crate::typing_rule_graph::TypingRuleRequiredSafety {
    use crate::candidate_contract::CandidateOrigin;
    use crate::typing_rule_graph::TypingRuleRequiredSafety;

    if alias.source == CorrectionDecisionSource::Deterministic {
        if let Some(rule) = crate::typing_rule_graph::find_typing_rule(&alias.source_id) {
            return rule.required_safety;
        }
        if alias.origin == CandidateOrigin::DeterministicTypo {
            return TypingRuleRequiredSafety::Experimental;
        }
    }
    if matches!(
        alias.origin,
        CandidateOrigin::LayoutThenTypo
            | CandidateOrigin::Completion
            | CandidateOrigin::L3Context
            | CandidateOrigin::Technical
    ) {
        return TypingRuleRequiredSafety::Experimental;
    }
    match alias.error_class {
        TypingErrorClass::CaseNoise => TypingRuleRequiredSafety::Strict,
        TypingErrorClass::WrongLayout
        | TypingErrorClass::PartialLayout
        | TypingErrorClass::MixedScript
        | TypingErrorClass::MissingLetter
        | TypingErrorClass::RepeatedLetter
        | TypingErrorClass::AdjacentTransposition
        | TypingErrorClass::BoundaryShift
        | TypingErrorClass::SplitWord
        | TypingErrorClass::GluedWords => TypingRuleRequiredSafety::Normal,
        TypingErrorClass::SparseInternalMultiOmission
        | TypingErrorClass::ExtraLetter
        | TypingErrorClass::LetterSubstitution
        | TypingErrorClass::CompositeTypo
        | TypingErrorClass::GrammarAgreement
        | TypingErrorClass::CompletionOnly
        | TypingErrorClass::TechnicalToken
        | TypingErrorClass::ProtectedToken
        | TypingErrorClass::Unknown => TypingRuleRequiredSafety::Experimental,
    }
}

pub(super) fn candidate_has_apply_authority(
    event: &TypingErrorEvent,
    candidate_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
    policy: TransitionDecisionPolicy,
    frame_bound_lexical_authority: bool,
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
    let verified_deterministic_typo_repair =
        verified_current_token_deterministic_typo_repair(event, candidate, evaluation);
    let learned_short_boundary_authority = signals.l3_phrase_milli >= CURRENT.l3_strong_milli
        || signals.l4_signed_milli >= CURRENT.l4_strong_milli
        || exact_positive_transition;
    let context_state_support = super::calibration::known_word_context_state_support(
        bayes.context_prior,
        signals.l3_phrase_milli,
        signals.l4_signed_milli,
    );
    let lexical_boundary_split_authority =
        exact_current_token_function_word_split(&event.original, &candidate.replacement)
            || verified_l2_boundary_target_grounding(event, candidate, evaluation);
    let lexical_boundary_split_is_ambiguous = lexical_boundary_split_authority
        && exact_boundary_split_has_clean_single_token_repair(event, candidate);
    let boundary_split_semantic_authority = (lexical_boundary_split_authority
        && !lexical_boundary_split_is_ambiguous)
        || context_state_support
        || signals.l3_pairwise_certified
        || exact_positive_transition;
    let verified_boundary_transition = verified_typed_boundary_transition(
        &evaluation.transition,
        boundary_split_semantic_authority,
    );
    if let Some(reason) = hard_structural_veto::hidden_state_rejection(signals) {
        if (hidden_rejection_deferred_to_verified_boundary(reason) && verified_boundary_transition)
            || (hidden_rejection_deferred_to_verified_l2_repair(reason)
                && verified_l2_center_repair)
            || (hidden_rejection_deferred_to_verified_deterministic_repair(reason)
                && verified_deterministic_typo_repair)
            || (frame_bound_lexical_authority
                && reason == "ambiguous"
                && signals.l4_hidden_certificate_valid
                && !signals.l4_hidden_selected_witnessed
                && !lattice_has_independent_transition_evidence(candidates, evaluations))
        {
            // L4 ambiguity protects lexical choice operators from guessing. A
            // verifier-proven boundary edit or strongly separated
            // dirty-surface -> lexical-center repair is already a typed edit
            // certificate; only exact negative memory may veto it later.
            // A validated lexical capability also survives advisory ambiguity.
            // Independent evidence anywhere in the lattice disables that
            // exception; all downstream safety and verifier checks still run.
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
    if verified_current_token_boundary_merge_split(event, candidate, evaluation)
        && close_single_token_repair_competes_with_boundary(
            event,
            candidate_index,
            candidates,
            evaluations,
        )
        && !context_state_support
        && !signals.l3_pairwise_certified
        && signals.l3_phrase_milli < CURRENT.l3_strong_milli
        && signals.l4_signed_milli < CURRENT.l4_strong_milli
        && !exact_positive_transition
    {
        debug_decision_reject(
            candidate,
            "boundary_competes_with_single_token_repair",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
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
    if candidate.closed_exact_layout_certificate().is_none() {
        let safety = observe_correction_safety(
            candidate,
            policy.correction_safety,
            CorrectionSafetyEvidence {
                operator_consensus: operator_consensus_authority,
                verified_l2_center_repair,
                l3_directional_pair: signals.l3_pairwise_certified,
                l4_exact_state: exact_positive_transition,
                high_precision_boundary: high_precision_boundary_shift,
            },
        );
        if !safety.allow_apply {
            debug_decision_reject(candidate, safety.reason, bayes.posterior, bayes.risk);
            return false;
        }
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
        && !verified_deterministic_typo_repair
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
    let strong_transition_support = context_state_support
        || verified_l2_center_repair
        || verified_deterministic_typo_repair
        || verified_boundary_transition
        || verified_layout_projection;
    let admission = admit_evaluated_hidden_transition(
        candidates.len(),
        source_role,
        context_state_support,
        operator_consensus_authority,
        boundary_split_semantic_authority,
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
    if self_referential_surface_drift
        && !external_learned_support
        && !operator_consensus_authority
        && !frame_bound_lexical_authority
    {
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

fn close_single_token_repair_competes_with_boundary(
    event: &TypingErrorEvent,
    boundary_index: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> bool {
    let boundary_evaluation = &evaluations[boundary_index];
    candidates.iter().enumerate().any(|(index, candidate)| {
        if index == boundary_index
            || !matches!(
                candidate.origin.source_role(),
                CorrectionSourceRole::L2Surface | CorrectionSourceRole::DeterministicTypo
            )
            || matches!(
                candidate.gate.action,
                CandidateGateAction::KeepOriginal | CandidateGateAction::Veto
            )
            || !matches!(
                candidate.error_class,
                TypingErrorClass::MissingLetter
                    | TypingErrorClass::LetterSubstitution
                    | TypingErrorClass::ExtraLetter
                    | TypingErrorClass::RepeatedLetter
                    | TypingErrorClass::AdjacentTransposition
            )
        {
            return false;
        }
        let evaluation = &evaluations[index];
        if verified_zero_loss_boundary_dominates(boundary_evaluation, evaluation) {
            return false;
        }
        if !evaluation.action.verifier_passed
            || evaluation.action.left_context_changed
            || evaluation.action.changed_tokens != 1
            || evaluation.action.edit_operator
                != verifier::EditTransitionOperator::ReplaceCurrentWord
        {
            return false;
        }
        let Some((original, replacement)) = current_and_replacement_words(event, candidate) else {
            return false;
        };
        cyrillic_letters_only(&original)
            && cyrillic_letters_only(&replacement)
            && replacement_has_clean_surface_certificate(candidate)
            && damerau_levenshtein(&original, &replacement) <= 1
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
            || evaluations[other_index]
                .signals
                .l4_owner_precedence_rejected()
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

fn lattice_has_independent_transition_evidence(
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> bool {
    candidates
        .iter()
        .zip(evaluations)
        .any(|(candidate, evaluation)| {
            let signals = &evaluation.signals;
            matches!(
                signals.l3_phrase_decision,
                L3ContextDisposition::Support | L3ContextDisposition::Suppress
            ) || signals.l3_pairwise_certified
                || (signals.l4_transition_state_specific
                    && (signals.l4_transition_attract_count > 0
                        || signals.l4_transition_repel_count > 0))
                || verified_operator_consensus_witness(candidate, evaluation)
        })
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
        || crate::russian_lexicon::has_clean_russian_surface_certificate(&original)
        || !crate::russian_lexicon::has_clean_russian_surface_certificate(&replacement)
    {
        return false;
    }
    let distance = damerau_levenshtein(&original, &replacement);
    let typed_geometry = distance <= 1
        || crate::text_metrics::sparse_internal_omission_count(&original, &replacement).is_some()
        || (candidate.error_class == TypingErrorClass::CompositeTypo && distance <= 3);
    typed_geometry
        && phase_center_separates_candidate(event, candidate_index, candidates, evaluations)
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
        || crate::russian_lexicon::has_clean_russian_surface_certificate(&original)
        || !crate::russian_lexicon::has_clean_russian_surface_certificate(&replacement)
    {
        return false;
    }
    crate::ru_typo::correct_hard_sign_typo(&original).as_deref() == Some(replacement.as_str())
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
    boundary_split_semantic_authority: bool,
    transition: &TypingTransition,
) -> TransitionAdmission {
    let exact_state_support = transition.l4_signed_signal.exact_positive();
    let verified_boundary_transition =
        verified_typed_boundary_transition(transition, boundary_split_semantic_authority);
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
        && !verified_boundary_transition
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

    if verified_typed_word_count_increasing_boundary_split(transition)
        && !boundary_split_semantic_authority
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "boundary_split_needs_semantic_authority",
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

fn verified_typed_boundary_transition(
    transition: &TypingTransition,
    boundary_split_semantic_authority: bool,
) -> bool {
    transition.evidence.verifier_passed
        && transition.evidence.origin == crate::candidate_contract::CandidateOrigin::Boundary
        && transition.evidence.edit_proof == crate::language_action::LanguageActionProof::Boundary
        && match transition.evidence.edit_operator {
            crate::text_edit::TransitionOperator::BoundaryShift => true,
            crate::text_edit::TransitionOperator::BoundaryMergeSplit => {
                boundary_split_semantic_authority
                    || transition.state_after_predicted.word_count
                        < transition.state_before.word_count
            }
            _ => false,
        }
}

fn verified_typed_word_count_increasing_boundary_split(transition: &TypingTransition) -> bool {
    verified_typed_boundary_transition(transition, true)
        && transition.evidence.edit_operator
            == crate::text_edit::TransitionOperator::BoundaryMergeSplit
        && transition.state_after_predicted.word_count > transition.state_before.word_count
}

fn verified_l2_boundary_target_grounding(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    if !candidate.has_l2_boundary_target_grounding()
        || candidate.source != CorrectionDecisionSource::Nanda
        || candidate.origin != crate::candidate_contract::CandidateOrigin::Boundary
        || !candidate.has_eligible_origin(crate::candidate_contract::CandidateOrigin::Boundary)
        || !evaluation.action.verifier_passed
        || evaluation.action.edit_operator
            != crate::text_edit::TransitionOperator::BoundaryMergeSplit
        || !crate::text_metrics::current_token_boundary_split(
            &event.original,
            &candidate.replacement,
        )
    {
        return false;
    }
    let original_words = crate::correction_core::normalized_correction_words(&event.original);
    let replacement_words =
        crate::correction_core::normalized_correction_words(&candidate.replacement);
    let Some(split_index) = original_words.len().checked_sub(1) else {
        return false;
    };
    let (Some(original), Some(left), Some(right)) = (
        original_words.get(split_index),
        replacement_words.get(split_index),
        replacement_words.get(split_index + 1),
    ) else {
        return false;
    };
    crate::nanda_wave::l2::ime_l2_boundary_target_evidence(original, &format!("{left} {right}"))
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
            if unsettled_single_token_competitor_cannot_block_verified_boundary(
                selected_evaluation,
                candidate,
                candidate_evaluation,
            ) {
                return false;
            }
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
            let verified_boundary_dominates =
                verified_zero_loss_boundary_dominates(candidate_evaluation, selected_evaluation);
            candidate_bayes.risk <= selected_bayes.risk
                && (candidate_signals.rank_score > selected_signals.rank_score
                    || ((structurally_dominates || verified_boundary_dominates)
                        && candidate_signals.rank_score + CURRENT.structural_rank_proximity
                            >= selected_signals.rank_score))
        })
}

fn unsettled_single_token_competitor_cannot_block_verified_boundary(
    boundary: &CandidateDecisionEvaluation,
    competitor: &UnifiedCorrectionCandidate,
    competitor_evaluation: &CandidateDecisionEvaluation,
) -> bool {
    if !boundary.action.verifier_passed
        || !matches!(
            boundary.action.edit_operator,
            verifier::EditTransitionOperator::BoundaryShift
                | verifier::EditTransitionOperator::BoundaryMergeSplit
        )
        || !competitor_evaluation.action.verifier_passed
        || competitor_evaluation.action.edit_operator
            != verifier::EditTransitionOperator::ReplaceCurrentWord
    {
        return false;
    }

    !replacement_has_clean_surface_certificate(competitor)
}

fn replacement_has_clean_surface_certificate(candidate: &UnifiedCorrectionCandidate) -> bool {
    last_replacement_word(&candidate.replacement).is_some_and(|replacement| {
        crate::russian_lexicon::has_clean_russian_surface_certificate(&replacement.to_lowercase())
    })
}

fn verified_zero_loss_boundary_dominates(
    boundary: &CandidateDecisionEvaluation,
    competitor: &CandidateDecisionEvaluation,
) -> bool {
    boundary.action.edit_operator == verifier::EditTransitionOperator::BoundaryShift
        && boundary.action.changed_tokens <= 2
        && verified_zero_loss_boundary_evidence(boundary, competitor)
}

#[cfg(test)]
pub(super) fn verified_zero_loss_boundary_dominates_for_test(
    boundary: &CandidateDecisionEvaluation,
    competitor: &CandidateDecisionEvaluation,
) -> bool {
    verified_zero_loss_boundary_dominates(boundary, competitor)
}

fn exact_boundary_split_has_clean_single_token_repair(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
) -> bool {
    crate::text_metrics::current_token_boundary_split(&event.original, &candidate.replacement) && {
        let original = event.current_word.to_lowercase();
        cyrillic_letters_only(&original)
            && crate::russian_typo_candidates::has_clean_single_damerau_edit_candidate(&original)
    }
}

fn verified_zero_loss_boundary_evidence(
    boundary: &CandidateDecisionEvaluation,
    competitor: &CandidateDecisionEvaluation,
) -> bool {
    boundary.action.verifier_passed
        && matches!(
            boundary.action.edit_operator,
            verifier::EditTransitionOperator::BoundaryShift
                | verifier::EditTransitionOperator::BoundaryMergeSplit
        )
        && boundary.explanation.lost_mass_milli == 0
        && competitor.explanation.lost_mass_milli > 0
        && boundary.explanation.preservation_milli > competitor.explanation.preservation_milli
        && boundary.explanation.operator_fit_milli >= competitor.explanation.operator_fit_milli
}

pub(super) fn exact_current_token_function_word_split(original: &str, replacement: &str) -> bool {
    if !crate::text_metrics::current_token_boundary_split(original, replacement) {
        return false;
    }
    let original_words = crate::correction_core::normalized_correction_words(original);
    let replacement_words = crate::correction_core::normalized_correction_words(replacement);
    let split_index = original_words.len() - 1;
    if crate::russian_lexicon::has_clean_russian_surface_certificate(&original_words[split_index]) {
        return false;
    }
    let left = &replacement_words[split_index];
    let right = &replacement_words[split_index + 1];
    (crate::phrase_lexicon::is_short_russian_function_word(left)
        && crate::phrase_lexicon::is_known_russian_phrase_part(right))
        || (crate::phrase_lexicon::is_short_russian_function_word(right)
            && crate::phrase_lexicon::is_known_russian_phrase_part(left))
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
        && strongest_lexical_competitor.is_none_or(|competitor| {
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
    strongest_competitor.is_none_or(|competitor| {
        selected_signal
            .l2_transition_phase_milli
            .saturating_sub(competitor)
            >= CURRENT.phase_competitor_gap_milli
    })
}

#[cfg(test)]
mod correction_safety_tests {
    use super::*;
    use crate::candidate_contract::CandidateOrigin;
    use crate::config::CorrectionSafety;
    use crate::typing_rule_graph::{ids, TypingRuleRequiredSafety};

    fn candidate(
        source: CorrectionDecisionSource,
        origin: CandidateOrigin,
        source_id: &str,
        error_class: TypingErrorClass,
    ) -> UnifiedCorrectionCandidate {
        UnifiedCorrectionCandidate::new(
            "исправлено ",
            source,
            origin,
            source_id,
            error_class,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "test_candidate",
            },
        )
    }

    fn observe(
        candidate: &UnifiedCorrectionCandidate,
        profile: CorrectionSafety,
        evidence: CorrectionSafetyEvidence,
    ) -> CorrectionSafetyObservation {
        observe_correction_safety(candidate, profile, evidence)
    }

    fn assert_profile_matrix(
        case_id: &str,
        candidate: &UnifiedCorrectionCandidate,
        evidence: CorrectionSafetyEvidence,
        expected: [bool; 3],
    ) -> [CorrectionSafetyObservation; 3] {
        let observations = [
            observe(candidate, CorrectionSafety::Strict, evidence),
            observe(candidate, CorrectionSafety::Normal, evidence),
            observe(candidate, CorrectionSafety::Experimental, evidence),
        ];
        assert_eq!(
            observations.map(|observation| observation.allow_apply),
            expected,
            "case_id={case_id} observations={observations:#?}"
        );
        observations
    }

    #[test]
    fn td112_fixed_pure_policy_corpus_matches_p01_through_p06() {
        let strict = candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            ids::DUPLICATE_LAYOUT_PREFIX,
            TypingErrorClass::WrongLayout,
        );
        assert_profile_matrix(
            "P01",
            &strict,
            CorrectionSafetyEvidence::default(),
            [true, true, true],
        );

        let normal = candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            ids::MISSING_LETTER,
            TypingErrorClass::MissingLetter,
        );
        assert_profile_matrix(
            "P02",
            &normal,
            CorrectionSafetyEvidence::default(),
            [false, true, true],
        );

        let experimental = candidate(
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "typed_policy_fixture",
            TypingErrorClass::LetterSubstitution,
        );
        assert_profile_matrix(
            "P03",
            &experimental,
            CorrectionSafetyEvidence::default(),
            [false, false, true],
        );
        assert_profile_matrix(
            "P04",
            &experimental,
            CorrectionSafetyEvidence {
                verified_l2_center_repair: true,
                ..CorrectionSafetyEvidence::default()
            },
            [false, true, true],
        );
        let two_domains = assert_profile_matrix(
            "P05",
            &experimental,
            CorrectionSafetyEvidence {
                l3_directional_pair: true,
                l4_exact_state: true,
                ..CorrectionSafetyEvidence::default()
            },
            [true, true, true],
        );
        assert_eq!(
            two_domains[0].observed_domains,
            L3_DIRECTIONAL_PAIR | L4_EXACT_STATE
        );

        let duplicate_lexical = assert_profile_matrix(
            "P06",
            &experimental,
            CorrectionSafetyEvidence {
                operator_consensus: true,
                verified_l2_center_repair: true,
                ..CorrectionSafetyEvidence::default()
            },
            [false, true, true],
        );
        assert!(duplicate_lexical
            .iter()
            .all(|observation| observation.observed_domains == LEXICAL_FIELD
                && observation.observed_domain_count == 1));
    }

    #[test]
    fn correction_safety_profile_gap_matrix_matches_contract() {
        for (tier, source_id, error_class, expected) in [
            (
                TypingRuleRequiredSafety::Strict,
                ids::DUPLICATE_LAYOUT_PREFIX,
                TypingErrorClass::WrongLayout,
                [true, true, true],
            ),
            (
                TypingRuleRequiredSafety::Normal,
                ids::MISSING_LETTER,
                TypingErrorClass::MissingLetter,
                [false, true, true],
            ),
            (
                TypingRuleRequiredSafety::Experimental,
                ids::SINGLE_LETTER_SUBSTITUTION,
                TypingErrorClass::LetterSubstitution,
                [false, false, true],
            ),
        ] {
            let candidate = candidate(
                CorrectionDecisionSource::Deterministic,
                CandidateOrigin::DeterministicTypo,
                source_id,
                error_class,
            );
            for (profile, expected_allow) in [
                CorrectionSafety::Strict,
                CorrectionSafety::Normal,
                CorrectionSafety::Experimental,
            ]
            .into_iter()
            .zip(expected)
            {
                let observation = observe(&candidate, profile, CorrectionSafetyEvidence::default());
                assert_eq!(observation.candidate_tier, tier);
                assert_eq!(observation.allow_apply, expected_allow);
            }
        }
    }

    #[test]
    fn correction_safety_domains_are_deduplicated_and_boundary_exclusive() {
        let mut lexical = candidate(
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "L2SurfaceMotifCell32",
            TypingErrorClass::LetterSubstitution,
        );
        lexical.merge_evidence(candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            ids::SINGLE_LETTER_SUBSTITUTION,
            TypingErrorClass::LetterSubstitution,
        ));
        let duplicated_lexical = observe(
            &lexical,
            CorrectionSafety::Strict,
            CorrectionSafetyEvidence {
                operator_consensus: true,
                verified_l2_center_repair: true,
                ..CorrectionSafetyEvidence::default()
            },
        );
        assert_eq!(
            duplicated_lexical.candidate_tier,
            TypingRuleRequiredSafety::Experimental
        );
        assert_eq!(duplicated_lexical.required_domains, 2);
        assert_eq!(duplicated_lexical.observed_domains, LEXICAL_FIELD);
        assert_eq!(duplicated_lexical.observed_domain_count, 1);
        assert!(!duplicated_lexical.allow_apply);
        assert_eq!(
            duplicated_lexical.reason,
            "correction_safety_requires_more_evidence"
        );

        let mut boundary = candidate(
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "BoundaryCell32",
            TypingErrorClass::GluedWords,
        );
        boundary.merge_evidence(candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::Boundary,
            ids::GLUED_PHRASE,
            TypingErrorClass::GluedWords,
        ));
        let boundary_observation = observe(
            &boundary,
            CorrectionSafety::Strict,
            CorrectionSafetyEvidence {
                operator_consensus: true,
                verified_l2_center_repair: true,
                high_precision_boundary: true,
                ..CorrectionSafetyEvidence::default()
            },
        );
        assert_eq!(boundary_observation.observed_domains, BOUNDARY_FIELD);
        assert_eq!(boundary_observation.observed_domain_count, 1);
        assert!(boundary_observation.allow_apply);
    }

    #[test]
    fn correction_safety_tier_precedence_and_unknown_fallback_are_total() {
        let registered_precedes_origin = candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::LayoutThenTypo,
            ids::MISSING_LETTER,
            TypingErrorClass::CompositeTypo,
        );
        assert_eq!(
            observe(
                &registered_precedes_origin,
                CorrectionSafety::Experimental,
                CorrectionSafetyEvidence::default(),
            )
            .candidate_tier,
            TypingRuleRequiredSafety::Normal
        );

        let unregistered_deterministic_typo = candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            "unregistered_test_alias",
            TypingErrorClass::CaseNoise,
        );
        assert_eq!(
            observe(
                &unregistered_deterministic_typo,
                CorrectionSafety::Experimental,
                CorrectionSafetyEvidence::default(),
            )
            .candidate_tier,
            TypingRuleRequiredSafety::Experimental
        );

        let mut merged = candidate(
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "unregistered_nanda_alias",
            TypingErrorClass::LetterSubstitution,
        );
        merged.merge_evidence(candidate(
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            ids::MISSING_LETTER,
            TypingErrorClass::MissingLetter,
        ));
        assert_eq!(
            observe(
                &merged,
                CorrectionSafety::Experimental,
                CorrectionSafetyEvidence::default(),
            )
            .candidate_tier,
            TypingRuleRequiredSafety::Normal
        );

        merged.evidence.clear();
        assert_eq!(
            observe(
                &merged,
                CorrectionSafety::Experimental,
                CorrectionSafetyEvidence::default(),
            )
            .candidate_tier,
            TypingRuleRequiredSafety::Experimental
        );
    }

    #[test]
    fn correction_safety_error_class_mapping_is_exhaustive() {
        for (error_class, expected_tier) in [
            (
                TypingErrorClass::CaseNoise,
                TypingRuleRequiredSafety::Strict,
            ),
            (
                TypingErrorClass::WrongLayout,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::PartialLayout,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::MixedScript,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::MissingLetter,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::RepeatedLetter,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::AdjacentTransposition,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::BoundaryShift,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::SplitWord,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::GluedWords,
                TypingRuleRequiredSafety::Normal,
            ),
            (
                TypingErrorClass::SparseInternalMultiOmission,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::ExtraLetter,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::LetterSubstitution,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::CompositeTypo,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::GrammarAgreement,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::CompletionOnly,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::TechnicalToken,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::ProtectedToken,
                TypingRuleRequiredSafety::Experimental,
            ),
            (
                TypingErrorClass::Unknown,
                TypingRuleRequiredSafety::Experimental,
            ),
        ] {
            let candidate = candidate(
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::L2Surface,
                "typed_test_alias",
                error_class,
            );
            assert_eq!(
                observe(
                    &candidate,
                    CorrectionSafety::Experimental,
                    CorrectionSafetyEvidence::default(),
                )
                .candidate_tier,
                expected_tier,
                "error_class={error_class:?}"
            );
        }

        for origin in [
            CandidateOrigin::LayoutThenTypo,
            CandidateOrigin::Completion,
            CandidateOrigin::L3Context,
            CandidateOrigin::Technical,
        ] {
            let candidate = candidate(
                CorrectionDecisionSource::Nanda,
                origin,
                "typed_test_alias",
                TypingErrorClass::CaseNoise,
            );
            assert_eq!(
                observe(
                    &candidate,
                    CorrectionSafety::Experimental,
                    CorrectionSafetyEvidence::default(),
                )
                .candidate_tier,
                TypingRuleRequiredSafety::Experimental,
                "origin={origin:?}"
            );
        }
    }

    #[test]
    fn correction_safety_distinct_domains_satisfy_two_tier_gap() {
        let candidate = candidate(
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "unregistered_nanda_alias",
            TypingErrorClass::CompositeTypo,
        );
        let observation = observe(
            &candidate,
            CorrectionSafety::Strict,
            CorrectionSafetyEvidence {
                verified_l2_center_repair: true,
                l3_directional_pair: true,
                l4_exact_state: true,
                ..CorrectionSafetyEvidence::default()
            },
        );
        assert_eq!(observation.required_domains, 2);
        assert_eq!(
            observation.observed_domains,
            LEXICAL_FIELD | L3_DIRECTIONAL_PAIR | L4_EXACT_STATE
        );
        assert_eq!(observation.observed_domain_count, 3);
        assert!(observation.allow_apply);
        assert_eq!(observation.reason, "correction_safety_admitted");
    }
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
