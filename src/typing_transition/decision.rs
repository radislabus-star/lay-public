use super::{action, verifier, TypingTransition};
use crate::correction_core::{
    bayes_score_for_candidate, explanation_for_candidate, CandidateGateAction,
    CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass, TypingErrorEvent,
    UnifiedCorrectionCandidate,
};
use crate::correction_source_contract::CorrectionSourceRole;
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::l4_goal_state::{derive_l4_scene_state, L4AllowedAction, L4SceneStateInput};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use crate::text_edit::{
    authorize_replacement_with_transition, tail_chars, LatentTextTransitionCandidate,
    TextReplacement, TextTransitionDecision, TextTransitionRejection, TransitionAudit,
    VisibleFieldState,
};
use crate::text_metrics::damerau_levenshtein;
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
        let action = authorize_replacement_with_transition(
            "ibus-committed-tail",
            1000,
            &original_text,
            &candidate.insert_text,
            plan.clone(),
            Some(candidate.source.source_id()),
            None,
            transition,
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

    pub(crate) fn select_apply_candidate(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        policy: TransitionDecisionPolicy,
    ) -> Option<UnifiedCorrectionCandidate> {
        candidates
            .iter()
            .filter(|candidate| candidate.gate.action == CandidateGateAction::Apply)
            .filter(|candidate| candidate_has_apply_authority(event, candidate, candidates, policy))
            .cloned()
            .max_by(|left, right| {
                candidate_decision_signals(event, left, candidates.len())
                    .rank_score
                    .total_cmp(
                        &candidate_decision_signals(event, right, candidates.len()).rank_score,
                    )
            })
    }

    pub(crate) fn authorize_gate(
        original: &str,
        replacement: &str,
        error_class: TypingErrorClass,
        origin: crate::correction_source_contract::CandidateOrigin,
        source_id: &str,
        provisional: CandidateGateDecision,
    ) -> CandidateGateDecision {
        if !matches!(
            provisional.action,
            CandidateGateAction::Eligible | CandidateGateAction::Apply
        ) {
            return provisional;
        }
        let event = TypingErrorEvent {
            original: original.to_string(),
            core: original.trim().to_string(),
            current_word: last_replacement_word(original).unwrap_or_default(),
            input_class: error_class,
        };
        let candidate = UnifiedCorrectionCandidate::new(
            replacement,
            crate::correction_core::CorrectionDecisionSource::Deterministic,
            source_id,
            error_class,
            provisional.clone(),
        );
        let admission = admit_hidden_transition(&event, &candidate, 1, origin.source_role(), false);
        if !admission.allow_apply {
            return CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: admission.reason,
            };
        }
        let transition = TypingTransition::from_candidate(
            original,
            replacement,
            error_class,
            origin,
            source_id,
            1,
        );
        if transition.evidence.left_context_changed && !transition.evidence.verifier_passed {
            return CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "edit_transition_not_verified",
            };
        }
        if transition.l4_signed_signal.negative {
            return CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "l4_negative_transition_memory",
            };
        }
        CandidateGateDecision {
            action: CandidateGateAction::Apply,
            reason: "transition_core_authorized",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransitionAdmission {
    pub(crate) allow_apply: bool,
    pub(crate) reason: &'static str,
}

fn candidate_has_apply_authority(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidates: &[UnifiedCorrectionCandidate],
    policy: TransitionDecisionPolicy,
) -> bool {
    let bayes = bayes_score_for_candidate(&event.original, candidate);
    let signals = candidate_decision_signals(event, candidate, candidates.len());
    let source_role = candidate.origin.source_role();
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
    let action = action::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    );
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
    let external_learned_support = bayes.usage_prior >= 0.080
        || bayes.context_prior >= 0.080
        || signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120;
    let strong_l2_peak_support =
        strong_l2_wave_peak_support(&signals) && !self_referential_surface_drift;
    let strong_learned_support = external_learned_support || strong_l2_peak_support;
    let strong_transition_support = strong_l2_wave_peak_transition_support(&signals)
        && !self_referential_surface_drift
        || (policy.l2_phase_apply
            && signals.l2_transition_phase_operator_promoted
            && signals.l2_transition_phase_verdict == "support"
            && signals.l2_transition_phase_milli >= signals.l2_transition_phase_threshold_milli)
        || signals.l3_phrase_milli >= 420
        || signals.l4_signed_milli >= 120;
    let admission = admit_hidden_transition(
        event,
        candidate,
        candidates.len(),
        source_role,
        strong_transition_support,
    );
    if !admission.allow_apply {
        debug_decision_reject(candidate, admission.reason, bayes.posterior, bayes.risk);
        return false;
    }
    if learned_candidate_shadowed_by_deterministic_owner(event, candidate, candidates, source_role)
    {
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
        CorrectionSourceRole::L2Surface | CorrectionSourceRole::Unknown => 0.34,
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
    let allowed = !matches!(
        source_role,
        CorrectionSourceRole::L2Surface
            | CorrectionSourceRole::L3Context
            | CorrectionSourceRole::Unknown
    ) || !better_non_apply_candidate_exists(event, candidate, candidates);
    if !allowed {
        debug_decision_reject(candidate, "better_non_apply", bayes.posterior, bayes.risk);
    }
    allowed
}

fn phase_managed_source(source_role: CorrectionSourceRole) -> bool {
    matches!(
        source_role,
        CorrectionSourceRole::DeterministicTypo
            | CorrectionSourceRole::L2Surface
            | CorrectionSourceRole::L3Context
            | CorrectionSourceRole::Unknown
    )
}

fn phase_policy_rejection(
    policy: TransitionDecisionPolicy,
    source_role: CorrectionSourceRole,
    package_loaded: bool,
    operator_present: bool,
    operator_promoted: bool,
    verdict: &str,
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
        "repel" => Some("l2_transition_phase_repel"),
        "unknown" => Some("l2_transition_phase_unknown"),
        _ => None,
    }
}

fn learned_candidate_shadowed_by_deterministic_owner(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidates: &[UnifiedCorrectionCandidate],
    source_role: CorrectionSourceRole,
) -> bool {
    if candidate.source != CorrectionDecisionSource::Nanda
        || !matches!(
            source_role,
            CorrectionSourceRole::L2Surface
                | CorrectionSourceRole::L3Context
                | CorrectionSourceRole::Unknown
        )
    {
        return false;
    }
    let Some(candidate_word) = last_replacement_word(&candidate.replacement) else {
        return false;
    };
    let original_word = event.current_word.to_lowercase();
    let candidate_distance = damerau_levenshtein(&original_word, &candidate_word.to_lowercase());

    candidates.iter().any(|other| {
        if other.source != CorrectionDecisionSource::Deterministic
            || other.gate.action != CandidateGateAction::Apply
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
        let transition = TypingTransition::from_candidate(
            &event.original,
            &other.replacement,
            other.error_class,
            other.origin,
            &other.source_id,
            candidates.len(),
        );
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

fn strong_l2_wave_peak_transition_support(signals: &CandidateDecisionSignals) -> bool {
    signals.l2_wave_peak_milli >= 780 && signals.l2_wave_peak_uncertainty_milli <= 300
}

pub(crate) fn admit_hidden_transition(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    strong_transition_support: bool,
) -> TransitionAdmission {
    let transition = TypingTransition::from_candidate(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
        &candidate.source_id,
        candidate_count,
    );

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
        && !known_word_drift_has_authority(source_role, candidate_count, strong_transition_support)
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
    candidate_count: usize,
    strong_learned_support: bool,
) -> bool {
    matches!(
        source_role,
        CorrectionSourceRole::Layout | CorrectionSourceRole::Boundary
    ) || (candidate_count >= 2 && strong_learned_support)
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

fn better_non_apply_candidate_exists(
    event: &TypingErrorEvent,
    selected: &UnifiedCorrectionCandidate,
    candidates: &[UnifiedCorrectionCandidate],
) -> bool {
    let selected_bayes = bayes_score_for_candidate(&event.original, selected);
    let selected_signals = candidate_decision_signals(event, selected, candidates.len());
    candidates.iter().any(|candidate| {
        if candidate == selected || candidate.gate.action == CandidateGateAction::Veto {
            return false;
        }
        let candidate_bayes = bayes_score_for_candidate(&event.original, candidate);
        let candidate_signals = candidate_decision_signals(event, candidate, candidates.len());
        candidate_bayes.risk <= selected_bayes.risk
            && candidate_signals.rank_score >= selected_signals.rank_score + 0.10
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
    pub(crate) l2_transition_phase_verdict: &'static str,
    pub(crate) l2_transition_phase_package_loaded: bool,
    pub(crate) l2_transition_phase_operator_present: bool,
    pub(crate) l2_transition_phase_operator_promoted: bool,
    pub(crate) l2_transition_phase_positive_centers: u8,
    pub(crate) l2_transition_phase_anti_centers: u8,
    pub(crate) l2_transition_phase_surfaces: u32,
    pub(crate) l3_phrase_milli: i16,
    pub(crate) l3_phrase_decision: &'static str,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: &'static str,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: &'static str,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
}

pub(crate) fn candidate_decision_signals(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
) -> CandidateDecisionSignals {
    let bayes = bayes_score_for_candidate(&event.original, candidate).posterior;
    let explanation = explanation_for_candidate(&event.original, candidate);
    let action = action::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    );
    let l3 = l3_phrase_signal(event, candidate);
    let l4_scene = l4_scene_signal(event, candidate_count);
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
    let phase =
        crate::nanda_wave::l2_transition_phase_readout(action.operator.as_str(), relation.atoms());
    let l4_signed = l4_signed_signal(event, candidate, relation.surface_key());
    let l2_wave_peak = l2_wave_peak_signal(event, candidate, candidate_count, phase);
    let rank_score = bayes
        + ((explanation.explanation_score_milli as f32 - 500.0) / 10_000.0)
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
        admit_hidden_transition, phase_policy_rejection, TransitionDecisionCore,
        TransitionDecisionPolicy,
    };
    use crate::correction_core::{
        CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass,
        TypingErrorEvent, UnifiedCorrectionCandidate,
    };
    use crate::correction_source_contract::{CandidateOrigin, CorrectionSourceRole};

    #[test]
    fn transition_core_blocks_unverified_left_context_apply() {
        let decision = TransitionDecisionCore::authorize_gate(
            "содержкой ",
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
            "L2SurfaceMotifCell32",
            CandidateGateDecision {
                action: CandidateGateAction::Apply,
                reason: "class_allows_apply",
            },
        );

        assert_eq!(decision.action, CandidateGateAction::SuggestOnly);
        assert_eq!(decision.reason, "latent_context_unverified");
    }

    #[test]
    fn transition_core_allows_verified_current_word_apply() {
        let decision = TransitionDecisionCore::authorize_gate(
            "провека ",
            "проверка ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::DeterministicTypo,
            "composite_ru_typo",
            CandidateGateDecision {
                action: CandidateGateAction::Apply,
                reason: "class_allows_apply",
            },
        );

        assert_eq!(decision.action, CandidateGateAction::Apply);
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
                "repel",
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
                "repel",
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
                "unknown",
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
                "repel"
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
                "unknown"
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
                "support"
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
                "repel"
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
                "unknown",
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
                "support",
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
            source_id,
            TypingErrorClass::CompositeTypo,
            CandidateGateDecision {
                action: CandidateGateAction::Apply,
                reason: "class_allows_apply",
            },
        )
    }

    #[test]
    fn hidden_state_blocks_single_weak_known_word_drift() {
        let admission = admit_hidden_transition(
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
        let admission = admit_hidden_transition(
            &event("звгрузи "),
            &candidate("загрузи ", "composite_ru_typo"),
            1,
            CorrectionSourceRole::DeterministicTypo,
            false,
        );

        assert!(admission.allow_apply, "{admission:?}");
    }

    #[test]
    fn hidden_state_blocks_context_imported_candidate_text() {
        let admission = admit_hidden_transition(
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
