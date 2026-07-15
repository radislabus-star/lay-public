use super::{action, verifier, TypingTransition};
use crate::candidate_contract::CorrectionSourceRole;
use crate::candidate_explanation::CandidateExplanation;
use crate::correction_bayes::BayesCandidateScore;
use crate::correction_core::{
    explanation_for_candidate, CandidateGateAction, CorrectionDecisionSource, TypingErrorClass,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::l4_goal_state::{derive_l4_scene_state, L4AllowedAction, L4SceneStateInput};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use crate::text_edit::{
    plan_decision_transition_edit, tail_chars, DecisionTransitionEditInput,
    LatentTextTransitionCandidate, TextReplacement, TextTransitionDecision,
    TextTransitionRejection, TransitionAudit, VisibleFieldState,
};
use crate::text_metrics::{damerau_levenshtein, score_to_milli};
use crate::transition_relation::{TransitionRelationAtoms, TransitionRelationInput};
use crate::word_reader::split_word_punctuation;

pub(crate) struct TransitionDecisionCore;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TransitionDecisionPolicy {
    pub(crate) l2_phase_apply: bool,
}

impl TransitionDecisionCore {
    pub(crate) fn decide_visible_text_transition(
        state: &VisibleFieldState,
        candidate: LatentTextTransitionCandidate,
    ) -> TextTransitionDecision {
        if candidate.delete_chars == 0 && candidate.insert_text.is_empty() {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::Noop,
                action: None,
            };
        }

        if !candidate.insert_text.is_empty() && state.visible_tail.ends_with(&candidate.insert_text)
        {
            return TextTransitionDecision::AlreadyApplied;
        }

        let original_text = tail_chars(&state.visible_tail, candidate.delete_chars as usize);
        if let Some(expected) = candidate.expected_tail.as_ref() {
            let focus_id = state.focus_id.as_deref();
            if !expected.matches_source_and_focus(candidate.source, focus_id)
                || !expected
                    .matches_current_suffix(&state.visible_tail, candidate.delete_chars as usize)
            {
                return TextTransitionDecision::Reject {
                    rejection: TextTransitionRejection::StaleVisibleTail {
                        expected: expected.expected_suffix.clone(),
                        actual: original_text,
                    },
                    action: None,
                };
            }
        }

        if state.external_selection_active {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleSurroundingText {
                    expected: original_text,
                    actual: state
                        .external_tail_before_cursor
                        .clone()
                        .unwrap_or_default(),
                },
                action: None,
            };
        }

        if state.external_state_present {
            let actual = state
                .external_tail_before_cursor
                .clone()
                .unwrap_or_default();
            if actual != original_text {
                return TextTransitionDecision::Reject {
                    rejection: TextTransitionRejection::StaleSurroundingText {
                        expected: original_text,
                        actual,
                    },
                    action: None,
                };
            }
        }

        let plan = TextReplacement {
            move_left: 0,
            backspaces: candidate.delete_chars,
            insert: candidate.insert_text.clone(),
            move_right: 0,
        };
        let transition = TransitionAudit::proven(
            candidate.intent.operator(),
            candidate.intent.proof(),
            true,
            false,
            1,
        );
        let receipt = DecisionTransitionReceipt::issue(
            original_text.clone(),
            candidate.insert_text.clone(),
            transition,
        );
        let action = plan_decision_transition_edit(
            DecisionTransitionEditInput {
                source: "ibus-committed-tail",
                confidence_milli: 1000,
                from_text: &original_text,
                to_text: &candidate.insert_text,
                plan: plan.clone(),
                selected_source_id: Some(candidate.source.source_id()),
                selected_error_class: None,
            },
            &receipt,
        );
        if !action.allow_apply() {
            return TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::UnsafeEdit {
                    reason: action.safety_reason(),
                },
                action: Some(action),
            };
        }

        TextTransitionDecision::Apply { plan, action }
    }

    pub(crate) fn evaluate_candidates(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        policy: TransitionDecisionPolicy,
    ) -> CandidateDecisionBatch {
        let usage = crate::nanda_wave::cached_usage_prior_snapshot();
        let l4_scene = l4_scene_signal(event, candidates.len());
        let context = CandidateDecisionContext {
            event,
            candidate_count: candidates.len(),
            usage: &usage,
            l4_scene,
        };
        let evaluations = candidates
            .iter()
            .map(|candidate| CandidateDecisionEvaluation::build(context, candidate))
            .collect::<Vec<_>>();
        if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
            for (candidate, evaluation) in candidates.iter().zip(&evaluations) {
                eprintln!(
                    "decision-core-candidate origin={:?} source_id={} class={} gate={:?} rank={:.3} usage={:.3} context={:.3} l3={} l4={} replacement={:?}",
                    candidate.origin,
                    candidate.source_id,
                    candidate.error_class.as_str(),
                    candidate.gate.action,
                    evaluation.signals.rank_score,
                    evaluation.bayes.usage_prior,
                    evaluation.bayes.context_prior,
                    evaluation.signals.l3_phrase_milli,
                    evaluation.signals.l4_signed_milli,
                    candidate.replacement
                );
            }
        }
        let selected_index = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.gate.action == CandidateGateAction::Eligible)
            .filter(|(index, _)| {
                candidate_has_apply_authority(event, *index, candidates, &evaluations, policy)
            })
            .max_by(|(left, _), (right, _)| {
                evaluations[*left]
                    .signals
                    .rank_score
                    .total_cmp(&evaluations[*right].signals.rank_score)
            })
            .map(|(index, _)| index);

        let selected_transition = selected_index.map(|index| {
            DecisionTransitionReceipt::from_selected_candidate(
                event,
                &candidates[index],
                &evaluations[index],
            )
        });

        CandidateDecisionBatch {
            evaluations,
            selected_index,
            selected_transition,
        }
    }
}

/// Opaque proof that `TransitionDecisionCore` selected this exact semantic
/// transition. Logs can describe the transition, but cannot manufacture this
/// receipt or use a serialized trace as apply authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecisionTransitionReceipt {
    original: String,
    replacement: String,
    transition: TransitionAudit,
}

impl DecisionTransitionReceipt {
    fn issue(original: String, replacement: String, transition: TransitionAudit) -> Self {
        Self {
            original,
            replacement,
            transition,
        }
    }

    fn from_selected_candidate(
        event: &TypingErrorEvent,
        candidate: &UnifiedCorrectionCandidate,
        evaluation: &CandidateDecisionEvaluation,
    ) -> Self {
        let action = evaluation.action;
        Self::issue(
            event.original.clone(),
            candidate.replacement.clone(),
            TransitionAudit::proven(
                action.edit_operator,
                action.edit_proof.into(),
                action.verifier_passed,
                action.left_context_changed,
                action.changed_tokens,
            ),
        )
    }

    pub(crate) fn projected_transition(
        &self,
        from_text: &str,
        to_text: &str,
    ) -> Option<TransitionAudit> {
        if !self.transition.is_verified()
            || !same_transition_projection(&self.original, &self.replacement, from_text, to_text)
        {
            return None;
        }
        Some(self.transition.clone())
    }

    pub(crate) fn diagnostic_transition(&self) -> TransitionAudit {
        self.transition.clone()
    }
}

fn same_transition_projection(
    original: &str,
    replacement: &str,
    from_text: &str,
    to_text: &str,
) -> bool {
    if original == from_text && replacement == to_text {
        return true;
    }
    let Some(original_prefix) = original.strip_suffix(from_text) else {
        return false;
    };
    replacement
        .strip_suffix(to_text)
        .is_some_and(|replacement_prefix| replacement_prefix == original_prefix)
}

#[derive(Debug, Clone)]
pub(crate) struct CandidateDecisionEvaluation {
    pub(crate) bayes: BayesCandidateScore,
    pub(crate) explanation: CandidateExplanation,
    pub(crate) action: action::CorrectionActionOperatorReport,
    pub(crate) signals: CandidateDecisionSignals,
    pub(crate) transition: TypingTransition,
}

#[derive(Clone, Copy)]
struct CandidateDecisionContext<'a> {
    event: &'a TypingErrorEvent,
    candidate_count: usize,
    usage: &'a crate::nanda_wave::UsagePriorSnapshot,
    l4_scene: L4SceneSignal,
}

struct CandidateSignalReadouts<'a> {
    context: CandidateDecisionContext<'a>,
    candidate: &'a UnifiedCorrectionCandidate,
    bayes: &'a BayesCandidateScore,
    explanation: CandidateExplanation,
    action: action::CorrectionActionOperatorReport,
    relation: &'a TransitionRelationAtoms,
    l4_memory: &'a crate::nanda_wave::l4_signed_memory::L4SignedMemorySignal,
}

impl CandidateDecisionEvaluation {
    fn build(
        context: CandidateDecisionContext<'_>,
        candidate: &UnifiedCorrectionCandidate,
    ) -> Self {
        let event = context.event;
        let usage = context.usage;
        let explanation = explanation_for_candidate(&event.original, candidate);
        let action = action::verify_action_operator(
            &event.original,
            &candidate.replacement,
            candidate.error_class,
            candidate.origin,
        );
        let relation = TransitionRelationAtoms::encode(
            &event.original,
            &candidate.replacement,
            TransitionRelationInput {
                action_operator: action.operator.as_str(),
                edit_operator: action.edit_operator.as_str(),
                proof: action.edit_proof.as_str(),
                verifier_passed: action.verifier_passed,
                left_context_changed: action.left_context_changed,
                changed_tokens: action.changed_tokens,
            },
        );
        let l4_memory = l4_signed_memory_readout(event, candidate, relation.surface_key(), usage);
        let bayes = crate::correction_bayes::bayes_score_candidate_with_readout(
            &event.original,
            &candidate.replacement,
            candidate.error_class.as_str(),
            candidate.origin,
            usage,
            &l4_memory,
        );
        let signals = candidate_decision_signals_from_readouts(CandidateSignalReadouts {
            context,
            candidate,
            bayes: &bayes,
            explanation,
            action,
            relation: &relation,
            l4_memory: &l4_memory,
        });
        let transition =
            TypingTransition::from_evaluated_candidate(super::EvaluatedTransitionInput {
                original: &event.original,
                replacement: &candidate.replacement,
                error_class: candidate.error_class,
                origin: candidate.origin,
                source_id: &candidate.source_id,
                candidate_count: context.candidate_count,
                action,
                l4_signed_signal: signals.l4_transition_signal(),
            });
        Self {
            bayes,
            explanation,
            action,
            signals,
            transition,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CandidateDecisionBatch {
    pub(crate) evaluations: Vec<CandidateDecisionEvaluation>,
    pub(crate) selected_index: Option<usize>,
    pub(crate) selected_transition: Option<DecisionTransitionReceipt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransitionAdmission {
    pub(crate) allow_apply: bool,
    pub(crate) reason: &'static str,
}

fn candidate_has_apply_authority(
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
    if source_role == CorrectionSourceRole::L3Context
        && signals.l3_phrase_milli < 420
        && signals.l4_signed_milli < 120
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
    let action = evaluation.action;
    let boundary_field = (action.edit_operator == verifier::EditTransitionOperator::BoundaryShift)
        .then(|| boundary_shift_field_readout(&event.original, &candidate.replacement))
        .flatten();
    let high_precision_boundary_shift = action.verifier_passed
        && action.edit_operator == verifier::EditTransitionOperator::BoundaryShift
        && boundary_shift_has_stable_token_mass(&candidate.replacement)
        && boundary_field.is_some_and(|readout| readout.has_direct_apply_mass());
    let learned_short_boundary_authority = signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120
        || exact_positive_transition;
    if action.edit_operator == verifier::EditTransitionOperator::BoundaryShift
        && !high_precision_boundary_shift
        && !learned_short_boundary_authority
    {
        debug_decision_reject(
            candidate,
            "ambiguous_short_boundary_shift",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if !action.verifier_passed
        && !matches!(
            source_role,
            CorrectionSourceRole::Layout
                | CorrectionSourceRole::Boundary
                | CorrectionSourceRole::DeterministicTypo
        )
    {
        debug_decision_reject(
            candidate,
            "unverified_transition",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    let self_referential_surface_drift = source_role == CorrectionSourceRole::L2Surface
        && short_same_length_surface_drift(&event.current_word, &candidate.replacement);
    let strong_l2_peak_support =
        strong_l2_wave_peak_support(signals) && !self_referential_surface_drift;
    let external_learned_support = bayes.usage_prior >= 0.080
        || bayes.context_prior >= 0.080
        || signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120;
    let contextual_transition_support = bayes.context_prior >= 0.080
        || signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120
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
    let strong_transition_support = signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120
        || (policy.l2_phase_apply
            && signals.l2_transition_phase_operator_promoted
            && signals.l2_transition_phase_verdict == crate::nanda_wave::PhaseVerdict::Support
            && signals.l2_transition_phase_milli >= signals.l2_transition_phase_threshold_milli
            && signals.l2_transition_phase_surfaces >= 3);
    let admission = admit_evaluated_hidden_transition(
        candidates.len(),
        source_role,
        strong_transition_support,
        &evaluation.transition,
    );
    if !admission.allow_apply {
        debug_decision_reject(candidate, admission.reason, bayes.posterior, bayes.risk);
        return false;
    }
    if learned_candidate_shadowed_by_deterministic_owner(
        event,
        candidate_index,
        candidates,
        evaluations,
        source_role,
    ) {
        debug_decision_reject(
            candidate,
            "deterministic_owner_gravity",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    if bayes.risk >= 0.62
        && !matches!(
            source_role,
            CorrectionSourceRole::Layout
                | CorrectionSourceRole::Boundary
                | CorrectionSourceRole::DeterministicTypo
        )
    {
        debug_decision_reject(candidate, "high_risk", bayes.posterior, bayes.risk);
        return false;
    }
    let posterior_floor = match source_role {
        CorrectionSourceRole::Layout => 0.0,
        CorrectionSourceRole::Boundary | CorrectionSourceRole::DeterministicTypo => 0.20,
        CorrectionSourceRole::L3Context => 0.28,
        CorrectionSourceRole::L2Surface => 0.34,
        CorrectionSourceRole::Completion | CorrectionSourceRole::Technical => 0.40,
    };
    if bayes.posterior < posterior_floor && !strong_learned_support {
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
        && competing_lexical_margin(event, candidate_index, candidates, evaluations) < 0.08
    {
        debug_decision_reject(
            candidate,
            "l2_composite_margin_low",
            bayes.posterior,
            bayes.risk,
        );
        return false;
    }
    let allowed =
        !stronger_unresolved_candidate_exists(event, candidate_index, candidates, evaluations);
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

fn original_tail_has_same_script_context(event: &TypingErrorEvent) -> bool {
    let words = crate::correction_core::normalized_correction_words(&event.original);
    let Some((current, left)) = words.split_last() else {
        return false;
    };
    if left.is_empty() {
        return false;
    }
    let current_is_ru = current.chars().all(is_russian_letter);
    let current_is_en = current.chars().all(|ch| ch.is_ascii_alphabetic());
    left.iter().rev().take(3).any(|word| {
        (current_is_ru && word.chars().all(is_russian_letter))
            || (current_is_en && word.chars().all(|ch| ch.is_ascii_alphabetic()))
    })
}

fn boundary_shift_has_stable_token_mass(replacement: &str) -> bool {
    let words = crate::correction_core::normalized_correction_words(replacement);
    let Some(pair) = words.get(words.len().saturating_sub(2)..) else {
        return false;
    };
    pair.len() == 2 && pair.iter().all(|word| word.chars().count() >= 4)
}

fn boundary_shift_field_readout(
    original: &str,
    replacement: &str,
) -> Option<crate::hot_field::HotBoundaryShiftReadout> {
    let original_words = crate::correction_core::normalized_correction_words(original);
    let replacement_words = crate::correction_core::normalized_correction_words(replacement);
    if original_words.len() != replacement_words.len() || original_words.len() < 2 {
        return None;
    }
    let original_pair = &original_words[original_words.len() - 2..];
    let replacement_pair = &replacement_words[replacement_words.len() - 2..];
    Some(
        crate::hot_field::HotFieldSnapshot::current().boundary_shift_readout(
            &original_pair[0],
            &original_pair[1],
            &replacement_pair[0],
            &replacement_pair[1],
        ),
    )
}

fn phase_managed_source(source_role: CorrectionSourceRole) -> bool {
    matches!(
        source_role,
        CorrectionSourceRole::DeterministicTypo
            | CorrectionSourceRole::L2Surface
            | CorrectionSourceRole::L3Context
    )
}

fn phase_policy_rejection(
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

fn strong_l2_wave_peak_support(signals: &CandidateDecisionSignals) -> bool {
    signals.l2_wave_peak_milli >= 650 && signals.l2_wave_peak_uncertainty_milli <= 450
}

fn admit_evaluated_hidden_transition(
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    strong_transition_support: bool,
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
            strong_transition_support,
            exact_state_support,
        )
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_known_word_drift_needs_state_proof",
        };
    }

    if !transition.l4_state_estimate.apply_allowed
        && transition.l4_state_estimate.desync_risk_milli >= 500
    {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_l4_state_desync_risk",
        };
    }

    if transition.l4_signed_signal.negative {
        return TransitionAdmission {
            allow_apply: false,
            reason: "latent_l4_negative_transition_memory",
        };
    }

    if source_role == CorrectionSourceRole::Layout && transition.evidence.verifier_passed {
        return TransitionAdmission {
            allow_apply: true,
            reason: "latent_layout_projection_admitted",
        };
    }

    TransitionAdmission {
        allow_apply: true,
        reason: "latent_transition_admitted",
    }
}

fn known_word_drift_has_authority(
    source_role: CorrectionSourceRole,
    _candidate_count: usize,
    strong_state_support: bool,
    exact_state_support: bool,
) -> bool {
    exact_state_support
        || matches!(
            source_role,
            CorrectionSourceRole::Layout | CorrectionSourceRole::Boundary
        )
        || strong_state_support
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
        && original.chars().all(is_russian_letter)
        && replacement.chars().all(is_russian_letter)
        && damerau_levenshtein(&original, &replacement) <= 2
}

fn last_replacement_word(text: &str) -> Option<String> {
    text.split_whitespace().rev().find_map(|token| {
        let (_, word, _) = split_word_punctuation(token);
        (!word.is_empty()).then(|| word.to_string())
    })
}

fn is_russian_letter(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
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
    let selected_is_proven_reversible = selected_action.verifier_passed
        && matches!(
            selected_action.edit_operator,
            verifier::EditTransitionOperator::BoundaryShift
                | verifier::EditTransitionOperator::BoundaryMergeSplit
        );
    if selected_is_proven_reversible {
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
            let structurally_dominates = preservation_gain >= 120
                && loss_reduction >= 120
                && candidate_explanation.operator_fit_milli
                    >= selected_explanation.operator_fit_milli;
            candidate_bayes.risk <= selected_bayes.risk
                && (candidate_signals.rank_score > selected_signals.rank_score
                    || (structurally_dominates
                        && candidate_signals.rank_score + 0.08 >= selected_signals.rank_score))
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
    if selected_signal.l2_wave_peak_milli >= 650
        && selected_signal.l2_wave_peak_uncertainty_milli <= 450
        && strongest_lexical_competitor.map_or(true, |competitor| {
            selected_signal
                .l2_wave_peak_positive_milli
                .saturating_sub(competitor)
                >= 100
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
            >= 10
    })
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CandidateDecisionSignals {
    pub(crate) rank_score: f32,
    pub(crate) rank_milli: i16,
    pub(crate) l2_wave_peak_milli: i16,
    pub(crate) l2_wave_peak_positive_milli: i16,
    pub(crate) l2_wave_peak_negative_milli: i16,
    pub(crate) l2_wave_peak_uncertainty_milli: i16,
    pub(crate) l2_wave_peak_reason: &'static str,
    pub(crate) l2_transition_phase_milli: i16,
    pub(crate) l2_transition_phase_threshold_milli: i16,
    pub(crate) l2_transition_phase_verdict: crate::nanda_wave::PhaseVerdict,
    pub(crate) l2_transition_phase_package_loaded: bool,
    pub(crate) l2_transition_phase_operator_present: bool,
    pub(crate) l2_transition_phase_operator_promoted: bool,
    pub(crate) l2_transition_phase_positive_centers: u8,
    pub(crate) l2_transition_phase_anti_centers: u8,
    pub(crate) l2_transition_phase_surfaces: u32,
    pub(crate) l3_phrase_milli: i16,
    pub(crate) l3_phrase_decision: L3ContextDisposition,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: L4AllowedAction,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: crate::nanda_wave::l4_signed_memory::L4SurfaceStatus,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
}

impl CandidateDecisionSignals {
    fn l4_transition_signal(&self) -> super::L4SignedTransitionSignal {
        let exact_positive = self.l4_transition_state_specific
            && self.l4_transition_attract_count > self.l4_transition_repel_count;
        let exact_negative = self.l4_transition_state_specific
            && self.l4_transition_repel_count > self.l4_transition_attract_count;
        super::L4SignedTransitionSignal {
            negative: exact_negative || (!exact_positive && self.l4_signed_milli <= -450),
            state_specific: self.l4_transition_state_specific,
            attract_count: self.l4_transition_attract_count,
            repel_count: self.l4_transition_repel_count,
        }
    }
}

fn candidate_decision_signals_from_readouts(
    readouts: CandidateSignalReadouts<'_>,
) -> CandidateDecisionSignals {
    let CandidateSignalReadouts {
        context,
        candidate,
        bayes,
        explanation,
        action,
        relation,
        l4_memory,
    } = readouts;
    let event = context.event;
    let l4_scene = context.l4_scene;
    let l3 = l3_phrase_signal(event, candidate);
    let phase =
        crate::nanda_wave::l2_transition_phase_readout(action.operator.as_str(), relation.atoms());
    let l4_signed = l4_signed_signal_from_memory(l4_memory);
    let l2_wave_peak = l2_wave_peak_signal(
        event,
        candidate,
        context.candidate_count,
        phase,
        context.usage,
    );
    let rank_score = bayes.posterior
        + ((explanation.explanation_score_milli as f32 - 500.0) / 2_000.0)
        + transition_rank_bonus(&action, candidate)
        + ((candidate.evidence_count().saturating_sub(1).min(3) as f32) * 0.025)
        + l2_wave_peak.rank_bonus
        + l3.rank_bonus
        + l4_scene.rank_bonus
        + l4_signed.rank_bonus;

    CandidateDecisionSignals {
        rank_score,
        rank_milli: score_to_milli(rank_score),
        l2_wave_peak_milli: score_to_milli(l2_wave_peak.signal),
        l2_wave_peak_positive_milli: l2_wave_peak.positive_milli,
        l2_wave_peak_negative_milli: l2_wave_peak.negative_milli,
        l2_wave_peak_uncertainty_milli: l2_wave_peak.uncertainty_milli,
        l2_wave_peak_reason: l2_wave_peak.reason,
        l2_transition_phase_milli: l2_wave_peak.transition_phase_milli,
        l2_transition_phase_threshold_milli: l2_wave_peak.transition_phase_threshold_milli,
        l2_transition_phase_verdict: l2_wave_peak.transition_phase_verdict,
        l2_transition_phase_package_loaded: l2_wave_peak.transition_phase_package_loaded,
        l2_transition_phase_operator_present: l2_wave_peak.transition_phase_operator_present,
        l2_transition_phase_operator_promoted: l2_wave_peak.transition_phase_operator_promoted,
        l2_transition_phase_positive_centers: l2_wave_peak.transition_phase_positive_centers,
        l2_transition_phase_anti_centers: l2_wave_peak.transition_phase_anti_centers,
        l2_transition_phase_surfaces: l2_wave_peak.transition_phase_surfaces,
        l3_phrase_milli: score_to_milli(l3.signal),
        l3_phrase_decision: l3.decision,
        l4_scene_milli: score_to_milli(l4_scene.signal),
        l4_scene_action: l4_scene.action,
        l4_scene_reason: l4_scene.reason,
        l4_signed_milli: score_to_milli(l4_signed.signal),
        l4_signed_reason: l4_signed.reason,
        l4_surface_status: l4_signed.surface_status,
        l4_transition_state_specific: l4_signed.transition_state_specific,
        l4_transition_attract_count: l4_signed.transition_attract_count,
        l4_transition_repel_count: l4_signed.transition_repel_count,
    }
}

include!("decision_signals.rs");

#[cfg(test)]
mod tests {
    use super::{
        admit_evaluated_hidden_transition, phase_policy_rejection, TransitionDecisionPolicy,
    };
    use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
    use crate::correction_core::{
        CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass,
        TypingErrorEvent, UnifiedCorrectionCandidate,
    };

    #[test]
    fn transition_admission_blocks_unverified_left_context() {
        let event = event("содержкой ");
        let candidate = UnifiedCorrectionCandidate::new(
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "L2SurfaceMotifCell32",
            TypingErrorClass::CompositeTypo,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "surface_candidate",
            },
        );
        let admission = admit(
            &event,
            &candidate,
            1,
            CorrectionSourceRole::L2Surface,
            false,
        );

        assert!(!admission.allow_apply);
        assert_eq!(admission.reason, "latent_context_unverified");
    }

    #[test]
    fn transition_admission_allows_verified_current_word() {
        let event = event("провека ");
        let candidate = candidate("проверка ", "composite_ru_typo");
        let admission = admit(
            &event,
            &candidate,
            1,
            CorrectionSourceRole::DeterministicTypo,
            true,
        );

        assert!(admission.allow_apply, "reason={}", admission.reason);
    }

    #[test]
    fn phase_cutover_blocks_learned_and_deterministic_typo_sources() {
        let disabled = TransitionDecisionPolicy {
            l2_phase_apply: false,
        };
        let enabled = TransitionDecisionPolicy {
            l2_phase_apply: true,
        };

        assert_eq!(
            phase_policy_rejection(
                disabled,
                CorrectionSourceRole::L2Surface,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Repel,
            ),
            None
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::L2Surface,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Repel,
            ),
            Some("l2_transition_phase_repel")
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::L2Surface,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Unknown,
            ),
            Some("l2_transition_phase_unknown")
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::DeterministicTypo,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Repel
            ),
            Some("l2_transition_phase_repel")
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::DeterministicTypo,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Unknown
            ),
            Some("l2_transition_phase_unknown")
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::DeterministicTypo,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Support
            ),
            None
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::Layout,
                true,
                true,
                true,
                crate::nanda_wave::PhaseVerdict::Repel
            ),
            None
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::L2Surface,
                false,
                false,
                false,
                crate::nanda_wave::PhaseVerdict::Unknown,
            ),
            Some("l2_transition_phase_package_missing")
        );
        assert_eq!(
            phase_policy_rejection(
                enabled,
                CorrectionSourceRole::L2Surface,
                true,
                true,
                false,
                crate::nanda_wave::PhaseVerdict::Support,
            ),
            Some("l2_transition_phase_shadow_only")
        );
    }

    fn event(text: &str) -> TypingErrorEvent {
        TypingErrorEvent {
            original: text.to_string(),
            core: text.trim().to_string(),
            current_word: text
                .split_whitespace()
                .last()
                .unwrap_or_default()
                .to_string(),
            input_class: TypingErrorClass::CompositeTypo,
        }
    }

    fn candidate(replacement: &str, source_id: &str) -> UnifiedCorrectionCandidate {
        UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            source_id,
            TypingErrorClass::CompositeTypo,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        )
    }

    fn admit(
        event: &TypingErrorEvent,
        candidate: &UnifiedCorrectionCandidate,
        candidate_count: usize,
        source_role: CorrectionSourceRole,
        strong_transition_support: bool,
    ) -> super::TransitionAdmission {
        let action = crate::typing_transition::action::verify_action_operator(
            &event.original,
            &candidate.replacement,
            candidate.error_class,
            candidate.origin,
        );
        let transition = crate::typing_transition::TypingTransition::from_evaluated_candidate(
            crate::typing_transition::EvaluatedTransitionInput {
                original: &event.original,
                replacement: &candidate.replacement,
                error_class: candidate.error_class,
                origin: candidate.origin,
                source_id: &candidate.source_id,
                candidate_count,
                action,
                l4_signed_signal: crate::typing_transition::L4SignedTransitionSignal {
                    negative: false,
                    state_specific: false,
                    attract_count: 0,
                    repel_count: 0,
                },
            },
        );
        admit_evaluated_hidden_transition(
            candidate_count,
            source_role,
            strong_transition_support,
            &transition,
        )
    }

    #[test]
    fn hidden_state_blocks_single_weak_known_word_drift() {
        let admission = admit(
            &event("мы можем "),
            &candidate("мы модем ", "composite_ru_typo"),
            1,
            CorrectionSourceRole::DeterministicTypo,
            false,
        );

        assert!(!admission.allow_apply);
        assert_eq!(
            admission.reason,
            "latent_known_word_drift_needs_state_proof"
        );
    }

    #[test]
    fn hidden_state_allows_unknown_to_known_typo_repair() {
        let admission = admit(
            &event("звгрузи "),
            &candidate("загрузи ", "composite_ru_typo"),
            1,
            CorrectionSourceRole::DeterministicTypo,
            false,
        );

        assert!(admission.allow_apply, "{admission:?}");
    }

    #[test]
    fn exact_state_proof_allows_single_learned_drift() {
        assert!(super::known_word_drift_has_authority(
            CorrectionSourceRole::DeterministicTypo,
            1,
            false,
            true,
        ));
    }

    #[test]
    fn hidden_state_blocks_context_imported_candidate_text() {
        let admission = admit(
            &event("можем "),
            &candidate("мы модем ", "composite_ru_typo"),
            2,
            CorrectionSourceRole::DeterministicTypo,
            true,
        );

        assert!(!admission.allow_apply);
        assert_eq!(admission.reason, "latent_context_unverified");
    }
}
