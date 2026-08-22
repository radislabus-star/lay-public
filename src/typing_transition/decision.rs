use super::{
    action,
    proposal_admission::{self, CandidateGateAction, CandidateGateDecision},
    verifier, TypingTransition,
};
use crate::candidate_contract::{CandidateOrigin, CandidateReadoutRoute, CorrectionSourceRole};
use crate::candidate_explanation::CandidateExplanation;
use crate::correction_bayes::BayesCandidateScore;
use crate::correction_core::{
    explanation_for_candidate, CorrectionDecisionSource, TypingErrorClass, TypingErrorEvent,
    UnifiedCorrectionCandidate,
};
use crate::keyboard::is_cyrillic_letter;
use crate::nanda_wave::l3_phrase_gate::L3PhraseGateDecision;
use crate::nanda_wave::l4_goal_state::L4AllowedAction;
use crate::nanda_wave::l4_hidden_state::{
    estimate_hidden_typing_state, predicted_state_id, L4HiddenCandidateInput, L4HiddenDisposition,
};
use crate::nanda_wave::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use crate::text_edit::TransitionAudit;
use crate::text_metrics::{damerau_levenshtein, score_to_milli};
use crate::transition_relation::{TransitionRelationAtoms, TransitionRelationInput};
use crate::word_reader::split_word_punctuation;
use rayon::prelude::*;
use std::cmp::Ordering;

pub(crate) struct TransitionDecisionCore;

#[derive(Clone, Copy)]
pub(crate) enum DecisionEvidenceMode<'a> {
    FullField(Option<&'a crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>),
    ClosedExact(&'a crate::exact_layout_authority::ExactLayoutContourCertificate),
    ClosedExactAbsent,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TransitionDecisionPolicy {
    pub(crate) l2_phase_apply: bool,
}

impl TransitionDecisionCore {
    pub(crate) fn decide_visible_text_transition(
        state: &crate::text_edit::VisibleFieldState,
        candidate: crate::text_edit::LatentTextTransitionCandidate,
    ) -> crate::text_edit::TextTransitionDecision {
        crate::text_edit::structural_verifier::verify_visible_text_transition(state, candidate)
    }

    pub(crate) fn admit_candidate_proposal(
        original: &str,
        replacement: &str,
        error_class: TypingErrorClass,
        origin: CandidateOrigin,
    ) -> CandidateGateDecision {
        proposal_admission::gate_candidate_with_origin(original, replacement, error_class, origin)
    }

    pub(crate) fn evaluate_candidates(
        event: &TypingErrorEvent,
        candidates: &[UnifiedCorrectionCandidate],
        policy: TransitionDecisionPolicy,
        mode: DecisionEvidenceMode<'_>,
    ) -> CandidateDecisionBatch {
        let timing_enabled = std::env::var_os("LAY_DECISION_CORE_TIMING").is_some();
        let started = std::time::Instant::now();
        match mode {
            DecisionEvidenceMode::ClosedExact(certificate) => {
                return evaluate_closed_exact(event, candidates, certificate, started);
            }
            DecisionEvidenceMode::ClosedExactAbsent => {
                return CandidateDecisionBatch::no_selection_with_started(started);
            }
            DecisionEvidenceMode::FullField(_) => {}
        }
        if candidates.is_empty() {
            return CandidateDecisionBatch::no_selection_with_started(started);
        }
        let usage = crate::nanda_wave::cached_usage_prior_snapshot();
        let usage_ready = std::time::Instant::now();
        let morphology_signals = l2_morphology_slot_signals(candidates);
        let morphology_ready = std::time::Instant::now();
        let replacements = candidates
            .iter()
            .map(|candidate| candidate.replacement.as_str())
            .collect::<Vec<_>>();
        let l3_reports = crate::nanda_wave::l3_phrase_gate::evaluate_default_candidates(
            &event.original,
            &replacements,
        );
        let l3_ready = std::time::Instant::now();
        let owned_l2_peak_context;
        let DecisionEvidenceMode::FullField(prepared_l2_peak_context) = mode else {
            unreachable!("closed exact modes returned above")
        };
        let l2_peak_context = if let Some(context) = prepared_l2_peak_context {
            context
        } else {
            owned_l2_peak_context =
                crate::nanda_wave::l2_wave_peak::prepare_correction_peak_context(&event.original);
            &owned_l2_peak_context
        };
        let peak_ready = std::time::Instant::now();
        let mut evaluations = candidates
            .par_iter()
            .zip(l3_reports.par_iter())
            .zip(morphology_signals.par_iter())
            .map(|((candidate, l3_report), morphology)| {
                CandidateDecisionEvaluation::build(
                    CandidateDecisionContext {
                        event,
                        candidate_count: candidates.len(),
                        usage: &usage,
                        l2_peak_context,
                        l3_report: l3_report.as_ref(),
                        morphology: *morphology,
                    },
                    candidate,
                )
            })
            .collect::<Vec<_>>();
        let evaluations_ready = std::time::Instant::now();
        settle_l2_morphology_competition(candidates, &mut evaluations);
        settle_transition_interference(candidates, &mut evaluations, policy);
        let interference_ready = std::time::Instant::now();
        settle_l4_hidden_state(event, candidates, &mut evaluations);
        let hidden_ready = std::time::Instant::now();
        if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
            for (candidate, evaluation) in candidates.iter().zip(&evaluations) {
                eprintln!(
                    "decision-core-candidate origin={:?} source_id={} class={} gate={:?} rank={:.3} posterior={:.3} risk={:.3} explain={} opfit={} lost={} field={} attract={} repel={} uncertainty={} phase_competition={} lexical_ready={} morphology={} morphology_disposition={} morphology_lemma={} morphology_slot={} morphology_competitors={} operator_consensus={} usage={:.3} context={:.3} l3={} l3_disposition={} l3_pairwise={} l3_relation={} l4={} l4_state_specific={} l4_attract={} l4_repel={} l4_phase={} l4_phase_supported={} cross_scene={} cross_margin={} cross_recommendation={} cross_auto_apply={} hidden={} hidden_classes={} hidden_selected={} hidden_probe={} hidden_certificate={} replacement={:?}",
                    candidate.origin,
                    candidate.source_id,
                    candidate.error_class.as_str(),
                    candidate.gate.action,
                    evaluation.signals.rank_score,
                    evaluation.bayes.posterior,
                    evaluation.bayes.risk,
                    evaluation.explanation.explanation_score_milli,
                    evaluation.explanation.operator_fit_milli,
                    evaluation.explanation.lost_mass_milli,
                    evaluation.signals.transition_field_milli,
                    evaluation.signals.transition_field_attraction_milli,
                    evaluation.signals.transition_field_repulsion_milli,
                    evaluation.signals.transition_field_uncertainty_milli,
                    evaluation
                        .signals
                        .transition_field_phase_competition_milli,
                    evaluation.signals.l2_lexical_phase_competition_ready,
                    evaluation.signals.l2_morphology_milli,
                    evaluation.signals.l2_morphology_disposition,
                    evaluation.signals.l2_morphology_lemma_id,
                    evaluation.signals.l2_morphology_target_feature_mask,
                    evaluation.signals.l2_morphology_competitors,
                    verified_operator_consensus_witness(candidate, evaluation),
                    evaluation.bayes.usage_prior,
                    evaluation.bayes.context_prior,
                    evaluation.signals.l3_phrase_milli,
                    evaluation.signals.l3_phrase_decision.as_str(),
                    evaluation.signals.l3_pairwise_certified,
                    evaluation.signals.l3_relation_class,
                    evaluation.signals.l4_signed_milli,
                    evaluation.signals.l4_transition_state_specific,
                    evaluation.signals.l4_transition_attract_count,
                    evaluation.signals.l4_transition_repel_count,
                    evaluation.signals.l4_phase_witness_milli,
                    evaluation.signals.l4_phase_witness_supported,
                    evaluation.signals.l4_cross_scene_disposition.as_str(),
                    evaluation.signals.l4_cross_scene_margin_milli,
                    evaluation.signals.l4_cross_scene_recommendation.as_str(),
                    evaluation.signals.l4_cross_scene_automatic_apply,
                    evaluation.signals.l4_hidden_disposition.as_str(),
                    evaluation.signals.l4_hidden_semantic_classes,
                    evaluation.signals.l4_hidden_selected_class,
                    evaluation.signals.l4_hidden_probe,
                    evaluation.signals.l4_hidden_certificate_valid,
                    candidate.replacement
                );
            }
        }
        let ranked_selected_index = candidates
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                apply_policy::producer_allows_authority_evaluation(
                    candidate.gate.action,
                    evaluations[*index].signals.l3_pairwise_certified,
                    evaluations[*index].transition.l4_signed_signal,
                ) || admission::suggest_boundary_allows_authority_evaluation(
                    event,
                    candidate,
                    &evaluations[*index],
                )
            })
            .filter(|(index, _)| {
                candidate_has_apply_authority(event, *index, candidates, &evaluations, policy)
            })
            .max_by(|(left, _), (right, _)| {
                compare_candidate_decision_order(*left, *right, candidates, &evaluations)
            })
            .map(|(index, _)| index);
        let selected_index = match retained_exact_disposition(event, candidates, &evaluations) {
            RetainedExactDisposition::Absent => ranked_selected_index,
            RetainedExactDisposition::Valid(index) => Some(index),
            RetainedExactDisposition::Invalid => None,
        };
        let selection_ready = std::time::Instant::now();

        let selected_transition = selected_index.map(|index| {
            DecisionTransitionReceipt::from_selected_candidate(
                event,
                &candidates[index],
                &evaluations[index],
            )
        });
        let finished = std::time::Instant::now();
        if timing_enabled {
            eprintln!(
                "lay_decision_core_timing usage_us={} morphology_us={} l3_us={} peak_us={} evaluations_us={} interference_us={} hidden_us={} selection_us={} receipt_us={} total_us={} candidates={}",
                usage_ready.duration_since(started).as_micros(),
                morphology_ready.duration_since(usage_ready).as_micros(),
                l3_ready.duration_since(morphology_ready).as_micros(),
                peak_ready.duration_since(l3_ready).as_micros(),
                evaluations_ready.duration_since(peak_ready).as_micros(),
                interference_ready.duration_since(evaluations_ready).as_micros(),
                hidden_ready.duration_since(interference_ready).as_micros(),
                selection_ready.duration_since(hidden_ready).as_micros(),
                finished.duration_since(selection_ready).as_micros(),
                finished.duration_since(started).as_micros(),
                candidates.len(),
            );
        }

        CandidateDecisionBatch {
            evaluations,
            selected_index,
            selected_transition,
            timing: CandidateDecisionTiming {
                l3_us: elapsed_us(l3_ready.duration_since(morphology_ready)),
                total_us: elapsed_us(finished.duration_since(started)),
            },
        }
    }
}

enum RetainedExactDisposition {
    Absent,
    Valid(usize),
    Invalid,
}

fn retained_exact_disposition(
    event: &TypingErrorEvent,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> RetainedExactDisposition {
    let mut retained = None;
    for (index, candidate) in candidates.iter().enumerate() {
        match &candidate.authority_evidence {
            crate::correction_core::CandidateAuthorityEvidence::None => {}
            crate::correction_core::CandidateAuthorityEvidence::Conflict => {
                return RetainedExactDisposition::Invalid;
            }
            crate::correction_core::CandidateAuthorityEvidence::ClosedExactLayout(certificate) => {
                if retained.is_some()
                    || !certificate.matches_candidate(&event.original, &candidate.replacement)
                    || !evaluations.get(index).is_some_and(|evaluation| {
                        closed_exact_candidate_is_verified(candidate, evaluation.action)
                    })
                {
                    return RetainedExactDisposition::Invalid;
                }
                retained = Some(index);
            }
        }
    }
    retained.map_or(
        RetainedExactDisposition::Absent,
        RetainedExactDisposition::Valid,
    )
}

fn evaluate_closed_exact(
    event: &TypingErrorEvent,
    candidates: &[UnifiedCorrectionCandidate],
    certificate: &crate::exact_layout_authority::ExactLayoutContourCertificate,
    started: std::time::Instant,
) -> CandidateDecisionBatch {
    let [candidate] = candidates else {
        return CandidateDecisionBatch::no_selection_with_started(started);
    };
    if candidate.closed_exact_layout_certificate() != Some(certificate)
        || !certificate.matches_candidate(&event.original, &candidate.replacement)
    {
        return CandidateDecisionBatch::no_selection_with_started(started);
    }
    let action = action::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    );
    if !closed_exact_candidate_is_verified(candidate, action) {
        return CandidateDecisionBatch::no_selection_with_started(started);
    }
    CandidateDecisionBatch {
        evaluations: Vec::new(),
        selected_index: Some(0),
        selected_transition: Some(DecisionTransitionReceipt::from_verified_action(
            event, candidate, action,
        )),
        timing: CandidateDecisionTiming {
            total_us: elapsed_us(started.elapsed()),
            ..CandidateDecisionTiming::default()
        },
    }
}

fn closed_exact_candidate_is_verified(
    candidate: &UnifiedCorrectionCandidate,
    action: action::CorrectionActionOperatorReport,
) -> bool {
    let target_evidence = candidate.common_target_evidence();
    exact_candidate_authority(&candidate.authority_evidence)
        && exact_source(candidate.source)
        && exact_origin(candidate.origin)
        && exact_source_role(candidate.origin.source_role())
        && exact_live_candidate_lane(live_candidate_lane_for_origin(candidate.origin))
        && exact_typing_candidate_family(typing_candidate_family_for_origin(candidate.origin))
        && exact_replacement_target_evidence(replacement_target_evidence_for_candidate(candidate))
        && exact_error_class(candidate.error_class)
        && exact_gate_action(candidate.gate.action)
        && exact_action_operator(action.operator)
        && exact_action_proof(action.proof)
        && exact_transition_operator(action.edit_operator)
        && exact_action_proof(action.edit_proof)
        && exact_transition_proof(action.edit_proof.into())
        && exact_transition_operator_kind(
            crate::transition_relation::TransitionOperatorKind::from_action_operator(
                action.operator.as_str(),
            ),
        )
        && exact_target_evidence_is_complete(&target_evidence)
        && action.verifier_required
        && action.verifier_passed
        && !action.left_context_changed
        && action.changed_tokens == 1
        && !candidate.has_authority_conflict()
}

pub(crate) const fn closed_exact_readout_route_preserves_retained_target(
    route: CandidateReadoutRoute,
) -> bool {
    match route {
        CandidateReadoutRoute::CanonicalL2Field | CandidateReadoutRoute::FullWave => true,
    }
}

fn exact_candidate_authority(
    authority: &crate::correction_core::CandidateAuthorityEvidence,
) -> bool {
    match authority {
        crate::correction_core::CandidateAuthorityEvidence::ClosedExactLayout(_) => true,
        crate::correction_core::CandidateAuthorityEvidence::None
        | crate::correction_core::CandidateAuthorityEvidence::Conflict => false,
    }
}

fn live_candidate_lane_for_origin(
    origin: CandidateOrigin,
) -> crate::typing_transition::live_candidate::LiveCandidateLane {
    use crate::typing_transition::live_candidate::LiveCandidateLane;
    match origin {
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo => {
            LiveCandidateLane::LayoutReplacement
        }
        CandidateOrigin::Boundary => LiveCandidateLane::BoundaryReplacement,
        CandidateOrigin::Completion => LiveCandidateLane::ExactCompletion,
        CandidateOrigin::L2Surface => LiveCandidateLane::LexicalRepairReplacement,
        CandidateOrigin::DeterministicTypo => LiveCandidateLane::CorrectedPrefixReplacement,
        CandidateOrigin::L3Context | CandidateOrigin::Technical => {
            LiveCandidateLane::GeneralReplacement
        }
    }
}

fn exact_live_candidate_lane(
    lane: crate::typing_transition::live_candidate::LiveCandidateLane,
) -> bool {
    use crate::typing_transition::live_candidate::LiveCandidateLane;
    match lane {
        LiveCandidateLane::LayoutReplacement => true,
        LiveCandidateLane::ExactCompletion
        | LiveCandidateLane::LexicalRepairReplacement
        | LiveCandidateLane::CorrectedPrefixReplacement
        | LiveCandidateLane::GeneralReplacement
        | LiveCandidateLane::BoundaryReplacement => false,
    }
}

fn typing_candidate_family_for_origin(
    origin: CandidateOrigin,
) -> crate::typing_candidate::TypingCandidateFamily {
    use crate::typing_candidate::TypingCandidateFamily;
    match origin {
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo => TypingCandidateFamily::Layout,
        CandidateOrigin::Boundary => TypingCandidateFamily::Structural,
        CandidateOrigin::Completion => TypingCandidateFamily::Exact,
        CandidateOrigin::DeterministicTypo => TypingCandidateFamily::Typo,
        CandidateOrigin::L2Surface | CandidateOrigin::L3Context => TypingCandidateFamily::Visual,
        CandidateOrigin::Technical => TypingCandidateFamily::Cleanup,
    }
}

fn exact_typing_candidate_family(family: crate::typing_candidate::TypingCandidateFamily) -> bool {
    use crate::typing_candidate::TypingCandidateFamily;
    match family {
        TypingCandidateFamily::Layout => true,
        TypingCandidateFamily::Exact
        | TypingCandidateFamily::Visual
        | TypingCandidateFamily::Structural
        | TypingCandidateFamily::Typo
        | TypingCandidateFamily::Cleanup
        | TypingCandidateFamily::Unknown => false,
    }
}

fn replacement_target_evidence_for_candidate(
    candidate: &UnifiedCorrectionCandidate,
) -> crate::typing_transition::target_evidence::ReplacementTargetEvidence {
    use crate::correction_core::CandidateAuthorityEvidence;
    use crate::typing_transition::target_evidence::ReplacementTargetEvidence;
    match &candidate.authority_evidence {
        CandidateAuthorityEvidence::ClosedExactLayout(_) => {
            ReplacementTargetEvidence::ExactLayoutProjection
        }
        CandidateAuthorityEvidence::None | CandidateAuthorityEvidence::Conflict => {
            ReplacementTargetEvidence::None
        }
    }
}

fn exact_replacement_target_evidence(
    evidence: crate::typing_transition::target_evidence::ReplacementTargetEvidence,
) -> bool {
    use crate::typing_transition::target_evidence::ReplacementTargetEvidence;
    match evidence {
        ReplacementTargetEvidence::ExactLayoutProjection => true,
        ReplacementTargetEvidence::None
        | ReplacementTargetEvidence::VerifiedLexicalEdit
        | ReplacementTargetEvidence::ContextBoundLexicalEdit
        | ReplacementTargetEvidence::VerifiedBoundary => false,
    }
}

fn exact_target_evidence_is_complete(
    evidence: &crate::typing_transition::target_evidence::TargetEvidenceSetV1,
) -> bool {
    exact_enumeration_state(evidence.state())
        && exact_completeness_scope_kind(evidence.scope().kind())
        && exact_incompleteness_reason(evidence.reason())
        && evidence.logical_count() == evidence.retained_count()
}

fn exact_enumeration_state(
    state: crate::typing_transition::target_evidence::EnumerationStateV1,
) -> bool {
    use crate::typing_transition::target_evidence::EnumerationStateV1;
    match state {
        EnumerationStateV1::Complete => true,
        EnumerationStateV1::Overflow | EnumerationStateV1::Failed => false,
    }
}

fn exact_completeness_scope_kind(
    scope: crate::typing_transition::target_evidence::CompletenessScopeKindV1,
) -> bool {
    use crate::typing_transition::target_evidence::CompletenessScopeKindV1;
    match scope {
        CompletenessScopeKindV1::WholePreparedField => true,
        CompletenessScopeKindV1::EditFootprintPartition
        | CompletenessScopeKindV1::RelationPartition => false,
    }
}

fn exact_incompleteness_reason(
    reason: crate::typing_transition::target_evidence::IncompletenessReasonV1,
) -> bool {
    use crate::typing_transition::target_evidence::IncompletenessReasonV1;
    match reason {
        IncompletenessReasonV1::None => true,
        IncompletenessReasonV1::StorageCapacity
        | IncompletenessReasonV1::WorkBudgetExceeded
        | IncompletenessReasonV1::UpstreamIncomplete
        | IncompletenessReasonV1::IntegrityFailure => false,
    }
}

fn exact_transition_proof(proof: crate::text_edit::TransitionProof) -> bool {
    use crate::text_edit::TransitionProof;
    match proof {
        TransitionProof::Layout => true,
        TransitionProof::Typo
        | TransitionProof::Boundary
        | TransitionProof::Completion
        | TransitionProof::Context
        | TransitionProof::Grammar
        | TransitionProof::VisibleState
        | TransitionProof::DecoderPlan
        | TransitionProof::ManualIntent
        | TransitionProof::UndoRecord
        | TransitionProof::NativeIntent
        | TransitionProof::Invariant => false,
    }
}

fn exact_transition_operator_kind(
    operator: crate::transition_relation::TransitionOperatorKind,
) -> bool {
    use crate::transition_relation::TransitionOperatorKind;
    match operator {
        TransitionOperatorKind::LayoutProjection => true,
        TransitionOperatorKind::AdjacentTransposition
        | TransitionOperatorKind::MissingLetterRepair
        | TransitionOperatorKind::RepeatedLetterRepair
        | TransitionOperatorKind::ExtraLetterRepair
        | TransitionOperatorKind::LetterSubstitution
        | TransitionOperatorKind::BoundarySplit
        | TransitionOperatorKind::BoundaryMerge
        | TransitionOperatorKind::AcceptCompletion
        | TransitionOperatorKind::CompositeTypo
        | TransitionOperatorKind::ContextChoice
        | TransitionOperatorKind::ManualToggle
        | TransitionOperatorKind::Other => false,
    }
}

fn exact_source(source: CorrectionDecisionSource) -> bool {
    match source {
        CorrectionDecisionSource::Deterministic => true,
        CorrectionDecisionSource::Nanda => false,
    }
}

fn exact_origin(origin: CandidateOrigin) -> bool {
    match origin {
        CandidateOrigin::Layout => true,
        CandidateOrigin::LayoutThenTypo
        | CandidateOrigin::Boundary
        | CandidateOrigin::Completion
        | CandidateOrigin::L2Surface
        | CandidateOrigin::L3Context
        | CandidateOrigin::DeterministicTypo
        | CandidateOrigin::Technical => false,
    }
}

fn exact_source_role(role: CorrectionSourceRole) -> bool {
    match role {
        CorrectionSourceRole::Layout => true,
        CorrectionSourceRole::Boundary
        | CorrectionSourceRole::Completion
        | CorrectionSourceRole::L2Surface
        | CorrectionSourceRole::L3Context
        | CorrectionSourceRole::DeterministicTypo
        | CorrectionSourceRole::Technical => false,
    }
}

fn exact_error_class(error_class: TypingErrorClass) -> bool {
    match error_class {
        TypingErrorClass::WrongLayout => true,
        TypingErrorClass::PartialLayout
        | TypingErrorClass::MixedScript
        | TypingErrorClass::MissingLetter
        | TypingErrorClass::SparseInternalMultiOmission
        | TypingErrorClass::ExtraLetter
        | TypingErrorClass::RepeatedLetter
        | TypingErrorClass::AdjacentTransposition
        | TypingErrorClass::LetterSubstitution
        | TypingErrorClass::CompositeTypo
        | TypingErrorClass::BoundaryShift
        | TypingErrorClass::SplitWord
        | TypingErrorClass::GluedWords
        | TypingErrorClass::CaseNoise
        | TypingErrorClass::GrammarAgreement
        | TypingErrorClass::CompletionOnly
        | TypingErrorClass::TechnicalToken
        | TypingErrorClass::ProtectedToken
        | TypingErrorClass::Unknown => false,
    }
}

fn exact_gate_action(gate: CandidateGateAction) -> bool {
    match gate {
        CandidateGateAction::Eligible => true,
        CandidateGateAction::SuggestOnly
        | CandidateGateAction::KeepOriginal
        | CandidateGateAction::Veto => false,
    }
}

fn exact_action_operator(operator: crate::language_action::LanguageActionOperator) -> bool {
    use crate::language_action::LanguageActionOperator;
    match operator {
        LanguageActionOperator::FlipLayout => true,
        LanguageActionOperator::KeepOriginal
        | LanguageActionOperator::SuggestOnly
        | LanguageActionOperator::FixTypo
        | LanguageActionOperator::FixTransposition
        | LanguageActionOperator::ReplaceLetter
        | LanguageActionOperator::RemoveExtraLetter
        | LanguageActionOperator::RestoreMissingLetter
        | LanguageActionOperator::NormalizeCase
        | LanguageActionOperator::FixGrammarForm
        | LanguageActionOperator::FixMixedLayout
        | LanguageActionOperator::CompleteWord
        | LanguageActionOperator::ShiftWordBoundary
        | LanguageActionOperator::SplitGluedWords
        | LanguageActionOperator::JoinBrokenWord
        | LanguageActionOperator::ApplyContextChoice => false,
    }
}

fn exact_action_proof(proof: crate::language_action::LanguageActionProof) -> bool {
    use crate::language_action::LanguageActionProof;
    match proof {
        LanguageActionProof::Layout => true,
        LanguageActionProof::None
        | LanguageActionProof::Typo
        | LanguageActionProof::Boundary
        | LanguageActionProof::Completion
        | LanguageActionProof::Context
        | LanguageActionProof::Grammar
        | LanguageActionProof::SafetyVeto => false,
    }
}

fn exact_transition_operator(operator: crate::text_edit::TransitionOperator) -> bool {
    use crate::text_edit::TransitionOperator;
    match operator {
        TransitionOperator::LayoutProjection => true,
        TransitionOperator::ReplaceCurrentWord
        | TransitionOperator::BoundaryShift
        | TransitionOperator::BoundaryMergeSplit
        | TransitionOperator::PhraseTokenRepair
        | TransitionOperator::SplitPreviousGluedAndRepairTail
        | TransitionOperator::Completion
        | TransitionOperator::VisibleTail
        | TransitionOperator::DecoderTail
        | TransitionOperator::ManualReplace
        | TransitionOperator::Undo
        | TransitionOperator::EnterAutocorrect
        | TransitionOperator::NativeReplace
        | TransitionOperator::Protected
        | TransitionOperator::Unknown => false,
    }
}

fn compare_candidate_decision_order(
    left: usize,
    right: usize,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
) -> Ordering {
    let left_eval = &evaluations[left];
    let right_eval = &evaluations[right];
    left_eval
        .signals
        .rank_score
        .total_cmp(&right_eval.signals.rank_score)
        .then_with(|| right_eval.bayes.risk.total_cmp(&left_eval.bayes.risk))
        .then_with(|| {
            left_eval
                .action
                .verifier_passed
                .cmp(&right_eval.action.verifier_passed)
        })
        .then_with(|| {
            right_eval
                .action
                .changed_tokens
                .cmp(&left_eval.action.changed_tokens)
        })
        .then_with(|| {
            candidates[right]
                .replacement
                .cmp(&candidates[left].replacement)
        })
}

mod apply_policy;
mod calibration;
mod hard_structural_veto;
mod interference;
mod live_field;
mod receipt;
pub(crate) use live_field::LiveFieldScoreInput;
pub(crate) use receipt::DecisionTransitionReceipt;

#[cfg(test)]
use apply_policy::{producer_allows_authority_evaluation, unresolved_competitor_blocks};

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
    l2_peak_context: &'a crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext,
    l3_report: Option<&'a crate::nanda_wave::l3_phrase_gate::L3PhraseGateReport>,
    morphology: L2MorphologySignal,
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
    pub(crate) timing: CandidateDecisionTiming,
}

impl CandidateDecisionBatch {
    pub(crate) fn no_selection() -> Self {
        Self {
            evaluations: Vec::new(),
            selected_index: None,
            selected_transition: None,
            timing: CandidateDecisionTiming::default(),
        }
    }

    fn no_selection_with_started(started: std::time::Instant) -> Self {
        Self {
            timing: CandidateDecisionTiming {
                total_us: elapsed_us(started.elapsed()),
                ..CandidateDecisionTiming::default()
            },
            ..Self::no_selection()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CandidateDecisionTiming {
    pub(crate) l3_us: u64,
    pub(crate) total_us: u64,
}

fn elapsed_us(duration: std::time::Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

mod admission;
use admission::candidate_has_apply_authority;
#[cfg(test)]
use admission::{admit_evaluated_hidden_transition, TransitionAdmission};
#[cfg(test)]
use calibration::known_word_drift_has_authority;
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CandidateDecisionSignals {
    non_field_rank_score: f32,
    l2_rank_energy: f32,
    l3_rank_energy: f32,
    l4_signed_rank_energy: f32,
    l2_transition_phase_margin_micro: i64,
    l2_transition_phase_threshold_micro: i64,
    l2_lexical_phase_margin_micro: i64,
    l2_lexical_phase_threshold_micro: i64,
    l2_lexical_phase_competition_ready: bool,
    pub(crate) rank_score: f32,
    pub(crate) rank_milli: i16,
    pub(crate) transition_field_milli: i16,
    pub(crate) transition_field_attraction_milli: i16,
    pub(crate) transition_field_repulsion_milli: i16,
    pub(crate) transition_field_uncertainty_milli: i16,
    pub(crate) transition_field_phase_competition_milli: i16,
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
    pub(crate) l2_morphology_milli: i16,
    pub(crate) l2_morphology_disposition: &'static str,
    pub(crate) l2_morphology_lemma_id: u32,
    pub(crate) l2_morphology_source_feature_mask: u32,
    pub(crate) l2_morphology_target_feature_mask: u32,
    pub(crate) l2_morphology_context_posterior_milli: u16,
    pub(crate) l2_morphology_slot_evidence_milli: i32,
    pub(crate) l2_morphology_joint_evidence_milli: u16,
    pub(crate) l2_morphology_competitors: u16,
    pub(crate) l2_morphology_generated: bool,
    pub(crate) l3_phrase_milli: i16,
    pub(crate) l3_phrase_decision: L3ContextDisposition,
    pub(crate) l3_relation_class: u64,
    pub(crate) l3_pairwise_certified: bool,
    pub(crate) l4_hidden_disposition: L4HiddenDisposition,
    pub(crate) l4_hidden_semantic_classes: u16,
    pub(crate) l4_hidden_unresolved_classes: u16,
    pub(crate) l4_hidden_selected_class: u64,
    pub(crate) l4_hidden_class_margin_milli: i16,
    pub(crate) l4_hidden_witness_count: u32,
    pub(crate) l4_hidden_ambiguity_authoritative: bool,
    pub(crate) l4_hidden_selected_witnessed: bool,
    pub(crate) l4_hidden_plan_commitment: u64,
    pub(crate) l4_hidden_receipts: u8,
    pub(crate) l4_hidden_probe: &'static str,
    pub(crate) l4_hidden_certificate_valid: bool,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: L4AllowedAction,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: crate::nanda_wave::l4_signed_memory::L4SurfaceStatus,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
    pub(crate) l4_phase_witness_milli: i16,
    pub(crate) l4_phase_witness_supported: bool,
    pub(crate) l4_phase_positive_centers: u8,
    pub(crate) l4_phase_negative_centers: u8,
    pub(crate) l4_cross_scene_package_loaded: bool,
    pub(crate) l4_cross_scene_profile_present: bool,
    pub(crate) l4_cross_scene_disposition:
        crate::nanda_wave::l4_cross_scene::L4CrossSceneDisposition,
    pub(crate) l4_cross_scene_recommendation:
        crate::nanda_wave::l4_cross_scene::L4CrossSceneRecommendation,
    pub(crate) l4_cross_scene_margin_milli: i16,
    pub(crate) l4_cross_scene_threshold_milli: i16,
    pub(crate) l4_cross_scene_pair_margin_milli: i16,
    pub(crate) l4_cross_scene_automatic_apply: bool,
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
    let l3 = l3_phrase_signal(candidate.error_class, context.l3_report);
    let morphology = context.morphology;
    let phase = crate::nanda_wave::l2_transition_phase_readout(
        action.operator.as_str(),
        relation.atoms(),
        &event.original,
        &candidate.replacement,
    );
    let l4_signed = l4_signed_signal_from_memory(l4_memory);
    let l2_wave_peak = l2_wave_peak_signal(
        candidate,
        context.candidate_count,
        phase,
        context.usage,
        context.l2_peak_context,
    );
    let cross_scene = l4_cross_scene_shadow_readout(event, candidate, relation, l3, l2_wave_peak);
    let non_field_rank_score = bayes.posterior
        + ((explanation.explanation_score_milli as f32 - 500.0) / 2_000.0)
        + transition_rank_bonus(&action, candidate)
        + ((candidate.evidence_count().saturating_sub(1).min(3) as f32) * 0.025);
    let transition_field =
        transition_interference_readout(l2_wave_peak, phase, l3, l4_signed, None);
    let rank_score = non_field_rank_score + transition_field.signal;

    CandidateDecisionSignals {
        non_field_rank_score,
        l2_rank_energy: l2_wave_peak.rank_energy,
        l3_rank_energy: l3.rank_energy,
        l4_signed_rank_energy: l4_signed.rank_energy,
        l2_transition_phase_margin_micro: phase.margin_micro,
        l2_transition_phase_threshold_micro: phase.threshold_micro,
        l2_lexical_phase_margin_micro: phase.lexical_margin_micro,
        l2_lexical_phase_threshold_micro: phase.lexical_threshold_micro,
        l2_lexical_phase_competition_ready: phase.lexical_competition_ready,
        rank_score,
        rank_milli: score_to_milli(rank_score),
        transition_field_milli: score_to_milli(transition_field.signal),
        transition_field_attraction_milli: score_to_milli(transition_field.attraction),
        transition_field_repulsion_milli: score_to_milli(transition_field.repulsion),
        transition_field_uncertainty_milli: score_to_milli(transition_field.uncertainty),
        transition_field_phase_competition_milli: score_to_milli(
            transition_field.phase_competition,
        ),
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
        l2_morphology_milli: morphology.signal_milli,
        l2_morphology_disposition: morphology.disposition,
        l2_morphology_lemma_id: morphology.lemma_id,
        l2_morphology_source_feature_mask: morphology.source_feature_mask,
        l2_morphology_target_feature_mask: morphology.target_feature_mask,
        l2_morphology_context_posterior_milli: morphology.context_posterior_milli,
        l2_morphology_slot_evidence_milli: morphology.slot_evidence_milli,
        l2_morphology_joint_evidence_milli: morphology.joint_evidence_milli,
        l2_morphology_competitors: morphology.competitors,
        l2_morphology_generated: morphology.generated,
        l3_phrase_milli: score_to_milli(l3.signal),
        l3_phrase_decision: l3.decision,
        l3_relation_class: l3.relation_class,
        l3_pairwise_certified: l3.pairwise_certified,
        l4_hidden_disposition: L4HiddenDisposition::Unobserved,
        l4_hidden_semantic_classes: 0,
        l4_hidden_unresolved_classes: 0,
        l4_hidden_selected_class: 0,
        l4_hidden_class_margin_milli: 0,
        l4_hidden_witness_count: 0,
        l4_hidden_ambiguity_authoritative: false,
        l4_hidden_selected_witnessed: false,
        l4_hidden_plan_commitment: 0,
        l4_hidden_receipts: 0,
        l4_hidden_probe: "none",
        l4_hidden_certificate_valid: false,
        l4_scene_milli: 0,
        l4_scene_action: L4AllowedAction::Wait,
        l4_scene_reason: "hidden_state_unobserved",
        l4_signed_milli: score_to_milli(l4_signed.signal),
        l4_signed_reason: l4_signed.reason,
        l4_surface_status: l4_signed.surface_status,
        l4_transition_state_specific: l4_signed.transition_state_specific,
        l4_transition_attract_count: l4_signed.transition_attract_count,
        l4_transition_repel_count: l4_signed.transition_repel_count,
        l4_phase_witness_milli: l4_signed.phase_witness_milli,
        l4_phase_witness_supported: l4_signed.phase_witness_supported,
        l4_phase_positive_centers: l4_signed.phase_positive_centers,
        l4_phase_negative_centers: l4_signed.phase_negative_centers,
        l4_cross_scene_package_loaded: cross_scene.package_loaded,
        l4_cross_scene_profile_present: cross_scene.profile_present,
        l4_cross_scene_disposition: cross_scene.disposition,
        l4_cross_scene_recommendation: cross_scene.recommendation,
        l4_cross_scene_margin_milli: cross_scene.margin_milli,
        l4_cross_scene_threshold_milli: cross_scene.threshold_milli,
        l4_cross_scene_pair_margin_milli: cross_scene.pair_margin_milli,
        l4_cross_scene_automatic_apply: cross_scene.recommendation.automatic_apply(),
    }
}

include!("decision_signals.rs");

#[cfg(test)]
mod tests;
