use super::{
    admit_evaluated_hidden_transition, producer_allows_authority_evaluation,
    unresolved_competitor_blocks,
};
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::config::CorrectionSafety;
use crate::correction_core::{
    CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, MorphologySlotEvidence,
    TypingErrorClass, TypingErrorEvent, UnifiedCorrectionCandidate,
};

#[test]
fn closed_exact_taxonomy_is_exhaustive_and_fail_closed() {
    use crate::candidate_contract::CandidateReadoutRoute;
    use crate::text_edit::TransitionProof;
    use crate::transition_relation::TransitionOperatorKind;
    use crate::typing_candidate::TypingCandidateFamily;
    use crate::typing_transition::live_candidate::{LiveCandidateLane, ReplacementTargetEvidence};
    use crate::typing_transition::target_evidence::{
        CompletenessScopeKindV1, EnumerationStateV1, IncompletenessReasonV1,
    };

    assert!(super::closed_exact_readout_route_preserves_retained_target(
        CandidateReadoutRoute::CanonicalL2Field
    ));
    assert!(super::closed_exact_readout_route_preserves_retained_target(
        CandidateReadoutRoute::FullWave
    ));

    for lane in [
        LiveCandidateLane::ExactCompletion,
        LiveCandidateLane::LexicalRepairReplacement,
        LiveCandidateLane::CorrectedPrefixReplacement,
        LiveCandidateLane::GeneralReplacement,
        LiveCandidateLane::BoundaryReplacement,
    ] {
        assert!(!super::exact_live_candidate_lane(lane));
    }
    assert!(super::exact_live_candidate_lane(
        LiveCandidateLane::LayoutReplacement
    ));

    for family in [
        TypingCandidateFamily::Exact,
        TypingCandidateFamily::Visual,
        TypingCandidateFamily::Structural,
        TypingCandidateFamily::Typo,
        TypingCandidateFamily::Cleanup,
        TypingCandidateFamily::Unknown,
    ] {
        assert!(!super::exact_typing_candidate_family(family));
    }
    assert!(super::exact_typing_candidate_family(
        TypingCandidateFamily::Layout
    ));

    for evidence in [
        ReplacementTargetEvidence::None,
        ReplacementTargetEvidence::VerifiedLexicalEdit,
        ReplacementTargetEvidence::ContextBoundLexicalEdit,
        ReplacementTargetEvidence::VerifiedBoundary,
    ] {
        assert!(!super::exact_replacement_target_evidence(evidence));
    }
    assert!(super::exact_replacement_target_evidence(
        ReplacementTargetEvidence::ExactLayoutProjection
    ));

    assert!(super::exact_enumeration_state(EnumerationStateV1::Complete));
    assert!(!super::exact_enumeration_state(
        EnumerationStateV1::Overflow
    ));
    assert!(!super::exact_enumeration_state(EnumerationStateV1::Failed));
    assert!(super::exact_completeness_scope_kind(
        CompletenessScopeKindV1::WholePreparedField
    ));
    assert!(!super::exact_completeness_scope_kind(
        CompletenessScopeKindV1::EditFootprintPartition
    ));
    assert!(!super::exact_completeness_scope_kind(
        CompletenessScopeKindV1::RelationPartition
    ));
    assert!(super::exact_incompleteness_reason(
        IncompletenessReasonV1::None
    ));
    for reason in [
        IncompletenessReasonV1::StorageCapacity,
        IncompletenessReasonV1::WorkBudgetExceeded,
        IncompletenessReasonV1::UpstreamIncomplete,
        IncompletenessReasonV1::IntegrityFailure,
    ] {
        assert!(!super::exact_incompleteness_reason(reason));
    }

    assert!(super::exact_transition_proof(TransitionProof::Layout));
    for proof in [
        TransitionProof::Typo,
        TransitionProof::Boundary,
        TransitionProof::Completion,
        TransitionProof::Context,
        TransitionProof::Grammar,
        TransitionProof::VisibleState,
        TransitionProof::DecoderPlan,
        TransitionProof::ManualIntent,
        TransitionProof::UndoRecord,
        TransitionProof::NativeIntent,
        TransitionProof::Invariant,
    ] {
        assert!(!super::exact_transition_proof(proof));
    }

    assert!(super::exact_transition_operator_kind(
        TransitionOperatorKind::LayoutProjection
    ));
    for operator in [
        TransitionOperatorKind::AdjacentTransposition,
        TransitionOperatorKind::MissingLetterRepair,
        TransitionOperatorKind::RepeatedLetterRepair,
        TransitionOperatorKind::ExtraLetterRepair,
        TransitionOperatorKind::LetterSubstitution,
        TransitionOperatorKind::BoundarySplit,
        TransitionOperatorKind::BoundaryMerge,
        TransitionOperatorKind::AcceptCompletion,
        TransitionOperatorKind::CompositeTypo,
        TransitionOperatorKind::ContextChoice,
        TransitionOperatorKind::ManualToggle,
        TransitionOperatorKind::Other,
    ] {
        assert!(!super::exact_transition_operator_kind(operator));
    }

    assert!(!super::exact_candidate_authority(
        &crate::correction_core::CandidateAuthorityEvidence::None
    ));
    assert!(!super::exact_candidate_authority(
        &crate::correction_core::CandidateAuthorityEvidence::Conflict
    ));

    let no_certificate = super::TransitionDecisionCore::evaluate_candidates(
        &event("ghbdtn "),
        &[],
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::ClosedExactAbsent,
    );
    assert!(no_certificate.selected_index.is_none());
    assert!(no_certificate.selected_transition.is_none());
}

fn morphology_evidence(
    lemma_id: u32,
    target_feature_mask: u32,
    context_positive_support: u32,
    context_alternative_support: u32,
    context_posterior_milli: u16,
    slot_evidence_milli: i32,
    joint_evidence_milli: u16,
) -> MorphologySlotEvidence {
    MorphologySlotEvidence {
        lemma_id,
        source_feature_mask: 1,
        target_feature_mask,
        context_positive_support,
        context_alternative_support,
        context_posterior_milli,
        slot_evidence_milli,
        joint_evidence_milli,
        generated: false,
    }
}

#[test]
fn morphology_same_lemma_slot_evidence_reranks_the_supported_ending() {
    let mut supported = l2_candidate(
        "вы принуждаете ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    supported.extend_morphology_slot_evidence([morphology_evidence(17, 10, 4, 0, 820, 700, 910)]);
    let mut alternative = l2_candidate(
        "вы принуждали ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    alternative.extend_morphology_slot_evidence([morphology_evidence(17, 11, 0, 4, 420, 300, 760)]);

    let signals = super::l2_morphology_slot_signals(&[supported, alternative]);

    assert_eq!(signals[0].disposition, "same_lemma_support");
    assert!(signals[0].rank_energy > 0.0);
    assert_eq!(signals[0].lemma_id, 17);
    assert_eq!(signals[0].target_feature_mask, 10);
    assert_eq!(signals[0].competitors, 1);
    assert_eq!(signals[1].disposition, "not_applicable");
    assert_eq!(signals[1].rank_energy, 0.0);
}

#[test]
fn morphology_slot_evidence_cannot_settle_cross_lemma_competition() {
    let mut left = l2_candidate(
        "вы принуждаете ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    left.extend_morphology_slot_evidence([morphology_evidence(17, 10, 4, 0, 900, 900, 950)]);
    let mut right = l2_candidate(
        "вы приближаете ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    right.extend_morphology_slot_evidence([morphology_evidence(23, 10, 4, 0, 300, 200, 700)]);

    let signals = super::l2_morphology_slot_signals(&[left, right]);

    assert!(signals
        .iter()
        .all(|signal| signal.disposition == "not_applicable" && signal.rank_energy == 0.0));
}

#[test]
fn morphology_two_context_supported_slots_remain_tied_despite_frequency_difference() {
    let mut frequent = l2_candidate(
        "ты принуждай ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    frequent.extend_morphology_slot_evidence([morphology_evidence(17, 20, 12, 2, 900, 900, 950)]);
    let mut less_frequent = l2_candidate(
        "ты принуждаешь ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    less_frequent
        .extend_morphology_slot_evidence([morphology_evidence(17, 21, 2, 12, 300, 200, 700)]);

    let signals = super::l2_morphology_slot_signals(&[frequent, less_frequent]);

    assert!(signals
        .iter()
        .all(|signal| signal.disposition == "not_applicable" && signal.rank_energy == 0.0));
}

#[test]
fn morphology_conflicting_same_lemma_axes_remain_tied() {
    let mut context_favored = l2_candidate(
        "они принуждают ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    context_favored
        .extend_morphology_slot_evidence([morphology_evidence(17, 20, 4, 0, 850, 200, 840)]);
    let mut slot_favored = l2_candidate(
        "они принуждали ",
        "CanonicalL2FieldSurface",
        TypingErrorClass::GrammarAgreement,
    );
    slot_favored
        .extend_morphology_slot_evidence([morphology_evidence(17, 21, 4, 0, 650, 800, 820)]);

    let signals = super::l2_morphology_slot_signals(&[context_favored, slot_favored]);

    assert!(signals
        .iter()
        .all(|signal| signal.disposition == "not_applicable" && signal.rank_energy == 0.0));
}

#[test]
fn morphology_budget_lift_preserves_number_geometry_inside_one_case_basin() {
    let singular = super::lift_preserving_relative_geometry(0.42, 0.61, 0.90);
    let plural = super::lift_preserving_relative_geometry(0.61, 0.61, 0.90);

    assert!((plural - singular - 0.19).abs() < f32::EPSILON);
    assert!((plural - 0.90).abs() < f32::EPSILON);
}

#[test]
fn productive_v90_tie_requires_common_l3_and_verified_transition() {
    let event = event("форма нужна форм ");
    let mut recurrent = l2_candidate(
        "форма нужна форма ",
        "ProductiveL2V90Surface",
        TypingErrorClass::GrammarAgreement,
    );
    recurrent.gate = CandidateGateDecision {
        action: CandidateGateAction::SuggestOnly,
        reason: "productive_v90_lattice_requires_common_l3",
    };
    recurrent.extend_morphology_slot_evidence([morphology_evidence(17, 10, 0, 0, 0, 0, 0)]);
    let mut alternative = l2_candidate(
        "форма нужна формы ",
        "ProductiveL2V90Surface",
        TypingErrorClass::GrammarAgreement,
    );
    alternative.gate = CandidateGateDecision {
        action: CandidateGateAction::SuggestOnly,
        reason: "productive_v90_lattice_requires_common_l3",
    };
    alternative.extend_morphology_slot_evidence([morphology_evidence(17, 11, 0, 0, 0, 0, 0)]);

    let candidates = [recurrent, alternative];
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
    );

    assert!(batch.evaluations[0].signals.l3_pairwise_certified);
    assert!(!batch.evaluations[1].signals.l3_pairwise_certified);
    assert!(batch.evaluations[0].action.verifier_passed);
    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
    assert!(batch.selected_transition.is_some());
}

#[test]
fn retained_two_edit_context_recurrence_reaches_verified_consumer_and_safety() {
    for (observed, target, alternative) in [
        ("рфрма", "форма", "ферма"),
        ("ткртина", "картина", "картона"),
    ] {
        assert_eq!(
            crate::text_metrics::damerau_levenshtein(observed, target),
            2
        );
        assert!(!crate::russian_lexicon::has_clean_russian_surface_certificate(observed));
        let event = event(&format!("{target} нужна {observed} "));
        let replacement = format!("{target} нужна {target} ");
        let alternative = format!("{target} нужна {alternative} ");
        let candidates = [&replacement, &alternative].map(|surface| {
            UnifiedCorrectionCandidate::new(
                surface,
                CorrectionDecisionSource::Deterministic,
                CandidateOrigin::DeterministicTypo,
                "retained_context_surface",
                TypingErrorClass::CompositeTypo,
                CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "candidate_requires_independent_context",
                },
            )
        });
        for (correction_safety, expected) in [
            (CorrectionSafety::Normal, Some(0)),
            (CorrectionSafety::Experimental, Some(0)),
            (CorrectionSafety::Strict, None),
        ] {
            let batch = super::TransitionDecisionCore::evaluate_candidates(
                &event,
                &candidates,
                super::TransitionDecisionPolicy {
                    correction_safety,
                    ..super::TransitionDecisionPolicy::default()
                },
                super::DecisionEvidenceMode::FullField(None),
            );
            assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
            assert_eq!(
                batch.selected_index, expected,
                "{observed}, {correction_safety:?}: {batch:#?}"
            );
            assert!(batch.evaluations[0].signals.l3_pairwise_certified);
            assert!(!batch.evaluations[1].signals.l3_pairwise_certified);
            if let Some(receipt) = batch.selected_transition {
                assert_eq!(batch.selected_candidate.unwrap().replacement, replacement);
                assert!(receipt
                    .projected_transition(&event.original, &replacement)
                    .is_some());
                assert!(receipt
                    .projected_transition(&event.original, &alternative)
                    .is_none());
            } else {
                assert_eq!(correction_safety, CorrectionSafety::Strict);
            }
        }
    }
}

#[test]
fn two_edit_context_evidence_cannot_bypass_the_actual_edit_verifier() {
    let event = event("форма нужна рфрма ");
    for (replacement, class) in [
        ("форма\nнужна форма ", TypingErrorClass::CompositeTypo),
        ("форма, нужна форма ", TypingErrorClass::CompositeTypo),
        ("форма нужна форма ", TypingErrorClass::ProtectedToken),
    ] {
        let candidate = UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            "retained_context_surface",
            class,
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "candidate_requires_independent_context",
            },
        );
        let reports = crate::nanda_wave::l3_phrase_gate::evaluate_default_candidates(
            &event.original,
            &[replacement],
        );
        assert!(reports[0].as_ref().unwrap().pairwise_certified);
        let proof = super::verifier::prove_edit_transition(
            &event.original,
            replacement,
            class,
            candidate.origin,
        );
        assert!(!proof.verified, "{proof:?}");
        for correction_safety in [
            CorrectionSafety::Normal,
            CorrectionSafety::Experimental,
            CorrectionSafety::Strict,
        ] {
            let batch = super::TransitionDecisionCore::evaluate_candidates(
                &event,
                std::slice::from_ref(&candidate),
                super::TransitionDecisionPolicy {
                    correction_safety,
                    ..super::TransitionDecisionPolicy::default()
                },
                super::DecisionEvidenceMode::FullField(None),
            );
            assert!(!batch.evaluations[0].action.verifier_passed, "{batch:#?}");
            assert!(batch.selected_index.is_none(), "{batch:#?}");
            assert!(batch.selected_transition.is_none());
        }
    }
}

#[test]
fn complete_context_competitors_and_clean_sources_keep_consumer_authority_closed() {
    for (original, replacement) in [
        ("форма ферма,дом рфрма ", "форма ферма,дом форма "),
        (
            "картина кртина нужна ткртина ",
            "картина кртина нужна картина ",
        ),
        (
            "картина ткртина нужна ткртина ",
            "картина ткртина нужна картина ",
        ),
        ("комета нужна комната ", "комета нужна комета "),
        ("сырье и сыр ", "сырье и сырье "),
    ] {
        let event = event(original);
        let mut candidate = candidate(replacement, "retained_context_surface");
        candidate.gate = CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "candidate_requires_independent_context",
        };
        for correction_safety in [
            CorrectionSafety::Normal,
            CorrectionSafety::Experimental,
            CorrectionSafety::Strict,
        ] {
            let batch = super::TransitionDecisionCore::evaluate_candidates(
                &event,
                std::slice::from_ref(&candidate),
                super::TransitionDecisionPolicy {
                    correction_safety,
                    ..super::TransitionDecisionPolicy::default()
                },
                super::DecisionEvidenceMode::FullField(None),
            );
            assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
            assert!(!batch.evaluations[0].signals.l3_pairwise_certified);
            assert!(
                batch.selected_index.is_none(),
                "{original}, {correction_safety:?}: {batch:#?}"
            );
            assert!(batch.selected_transition.is_none());
        }
    }
}

fn bound_lexical_advisory_fixture() -> (
    TypingErrorEvent,
    [UnifiedCorrectionCandidate; 2],
    crate::nanda_wave::l2_field::LexicalAuthorityEvaluationContextV1,
    Vec<super::CandidateDecisionEvaluation>,
) {
    let (event, target, context) = crate::nanda_wave::l2_field::test_bound_lexical_pipeline(
        "форм",
        "форма",
        TypingErrorClass::MissingLetter,
    );
    let rival = UnifiedCorrectionCandidate::new(
        "нужна формы ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L3Context,
        "context_candidate",
        TypingErrorClass::MissingLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    );
    let candidates = [target, rival];
    assert_eq!(
        context.admissions_for_event(&event, &candidates),
        [true, false]
    );
    let batch = super::TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        &candidates,
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
        &context,
    );
    for (candidate, evaluation) in candidates.iter().zip(&batch.evaluations) {
        assert!(evaluation.action.verifier_passed);
        assert!(!evaluation.signals.l3_pairwise_certified);
        assert!(!matches!(
            evaluation.signals.l3_phrase_decision,
            super::L3ContextDisposition::Support | super::L3ContextDisposition::Suppress
        ));
        assert!(!super::verified_operator_consensus_witness(
            candidate, evaluation
        ));
        assert_eq!(evaluation.signals.l4_transition_attract_count, 0);
        assert_eq!(evaluation.signals.l4_transition_repel_count, 0);
    }
    (event, candidates, context, batch.evaluations)
}

fn project_advisory_hidden_ambiguity(
    event: &TypingErrorEvent,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &mut [super::CandidateDecisionEvaluation],
    phase_margin: Option<i16>,
) {
    // Keep real lexical capabilities, action verification, ranking and safety
    // inputs. Supply an unresolved L4 evidence domain through its real resolver;
    // this is a controlled adapter probe, not a simulated candidate evaluator.
    let inputs = candidates
        .iter()
        .zip(evaluations.iter())
        .map(|(candidate, evaluation)| super::L4HiddenCandidateInput {
            predicted_state: super::predicted_state_id(
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
            context_support: false,
            pairwise_context_witness: false,
            eligible: candidate.gate.action == CandidateGateAction::Eligible,
            witness_attract: 0,
            witness_repel: 0,
            witness_state_specific: false,
            phase_witness_milli: phase_margin.unwrap_or_default(),
            phase_witness_supported: phase_margin.is_some(),
            operator_consensus_witness: false,
        })
        .collect::<Vec<_>>();
    let readouts = super::estimate_hidden_typing_state(&inputs);
    for (evaluation, readout) in evaluations.iter_mut().zip(readouts) {
        assert_eq!(readout.disposition, super::L4HiddenDisposition::Ambiguous);
        assert!(readout.certificate_valid);
        assert_eq!(readout.ambiguity_authoritative, phase_margin.is_some());
        assert_eq!(readout.semantic_classes, 2);
        let signals = &mut evaluation.signals;
        signals.l4_phase_witness_milli = phase_margin.unwrap_or_default();
        signals.l4_phase_witness_supported = phase_margin.is_some();
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
        signals.l4_scene_milli = 0;
        signals.l4_scene_action = super::L4AllowedAction::Wait;
        signals.l4_scene_reason = readout.disposition.as_str();
    }
}

#[test]
fn frame_bound_lexical_authority_survives_only_advisory_l4_ambiguity() {
    let (event, candidates, context, baseline) = bound_lexical_advisory_fixture();
    let mut observations = Vec::new();
    for phase_margin in [None, Some(0), Some(-600), Some(600)] {
        let mut evaluations = baseline.clone();
        project_advisory_hidden_ambiguity(&event, &candidates, &mut evaluations, phase_margin);
        let admissions = context.admissions_for_event(&event, &candidates);
        assert_eq!(admissions, [true, false]);
        for (profile, expected) in [
            (CorrectionSafety::Normal, true),
            (CorrectionSafety::Experimental, true),
            (CorrectionSafety::Strict, false),
        ] {
            let admission = super::authority_lane_allows_apply(
                &event,
                0,
                &candidates,
                &evaluations,
                super::TransitionDecisionPolicy {
                    l2_phase_apply: false,
                    correction_safety: profile,
                },
                candidates[0].clone(),
                admissions[0],
            );
            observations.push((phase_margin, profile, admission.is_some(), expected));
            if let Some(admission) = admission {
                assert_eq!(admission.candidate.replacement, "нужна форма ");
                assert!(admission
                    .candidate
                    .frame_bound_lexical_capability()
                    .is_some());
                assert!(admission.evaluation.action.verifier_passed);
                let receipt = super::DecisionTransitionReceipt::from_selected_candidate(
                    &event,
                    &admission.candidate,
                    &admission.evaluation,
                );
                assert!(receipt
                    .projected_transition(&event.original, &candidates[0].replacement)
                    .is_some());
                assert!(receipt
                    .projected_transition(&event.original, &candidates[1].replacement)
                    .is_none());
            }
        }
    }
    assert!(
        observations.iter().all(|row| row.2 == row.3),
        "{observations:#?}"
    );
}

#[test]
fn frame_bound_lexical_ambiguity_deferral_checks_independent_evidence_on_every_candidate() {
    let (event, candidates, context, mut baseline) = bound_lexical_advisory_fixture();
    project_advisory_hidden_ambiguity(&event, &candidates, &mut baseline, Some(0));
    for index in 0..candidates.len() {
        for evidence in [
            "l3_support",
            "l3_suppress",
            "l3_pairwise",
            "exact_positive",
            "exact_negative",
            "exact_tied",
            "operator_consensus",
        ] {
            let mut candidates = candidates.clone();
            let mut evaluations = baseline.clone();
            let signals = &mut evaluations[index].signals;
            // Hold the real unresolved L4 readout at the adapter boundary. A
            // witness elsewhere in the full evaluated lattice must disable the
            // exception even when it has not selected a hidden state.
            match evidence {
                "l3_support" => signals.l3_phrase_decision = super::L3ContextDisposition::Support,
                "l3_suppress" => signals.l3_phrase_decision = super::L3ContextDisposition::Suppress,
                "l3_pairwise" => signals.l3_pairwise_certified = true,
                "exact_positive" | "exact_negative" | "exact_tied" => {
                    signals.l4_transition_state_specific = true;
                    signals.l4_transition_attract_count = u32::from(evidence != "exact_negative");
                    signals.l4_transition_repel_count = u32::from(evidence != "exact_positive");
                }
                "operator_consensus" => {
                    let replacement = candidates[index].replacement.clone();
                    candidates[index].merge_evidence(l2_candidate(
                        &replacement,
                        "surface_candidate",
                        TypingErrorClass::MissingLetter,
                    ));
                    candidates[index].merge_evidence(UnifiedCorrectionCandidate::new(
                        &replacement,
                        CorrectionDecisionSource::Deterministic,
                        CandidateOrigin::DeterministicTypo,
                        "missing_letter",
                        TypingErrorClass::MissingLetter,
                        CandidateGateDecision {
                            action: CandidateGateAction::Eligible,
                            reason: "class_allows_apply",
                        },
                    ));
                    signals.l2_wave_peak_milli = super::calibration::CURRENT.l2_peak_milli;
                    signals.l2_wave_peak_uncertainty_milli = 0;
                    signals.l2_transition_phase_operator_promoted = false;
                    assert!(super::verified_operator_consensus_witness(
                        &candidates[index],
                        &evaluations[index]
                    ));
                    assert!(!super::certified_operator_consensus(
                        &event,
                        &candidates[index],
                        &evaluations[index]
                    ));
                }
                _ => unreachable!(),
            }
            evaluations[index].transition.l4_signed_signal =
                evaluations[index].signals.l4_transition_signal();
            let validated = context.admissions_for_event(&event, &candidates);
            assert_eq!(validated, [true, false], "{evidence} at {index}");
            let admission = super::authority_lane_allows_apply(
                &event,
                0,
                &candidates,
                &evaluations,
                super::TransitionDecisionPolicy::default(),
                candidates[0].clone(),
                validated[0],
            );
            assert!(admission.is_none(), "{evidence} at {index}");
            assert_eq!(candidates[0].replacement, "нужна форма ");
            assert!(evaluations[0].action.verifier_passed);
        }
    }
}

#[test]
fn frame_bound_lexical_ambiguity_deferral_preserves_frame_and_verifier_failures() {
    use crate::nanda_wave::l2_field::LexicalAuthorityEvaluationContextV1;

    let (event, candidates, context, mut evaluations) = bound_lexical_advisory_fixture();
    project_advisory_hidden_ambiguity(&event, &candidates, &mut evaluations, Some(0));
    let (_, _, foreign_context) = crate::nanda_wave::l2_field::test_bound_lexical_pipeline(
        "форм",
        "форма",
        TypingErrorClass::MissingLetter,
    );
    let mut changed_event = event.clone();
    changed_event.original = "другая форм ".to_string();
    let mut changed_candidate = candidates[0].clone();
    changed_candidate.replacement = candidates[1].replacement.clone();
    let missing_context = LexicalAuthorityEvaluationContextV1::NotConsulted;
    let validations = [
        (
            "missing_context",
            missing_context.admissions_for_event(&event, &candidates)[0],
        ),
        (
            "foreign_settlement",
            foreign_context.admissions_for_event(&event, &candidates)[0],
        ),
        (
            "changed_event",
            context.admissions_for_event(&changed_event, &candidates)[0],
        ),
        (
            "changed_surface",
            context.admissions_for_event(&event, &[changed_candidate])[0],
        ),
        (
            "expired_lease",
            context.admits_candidate_at(&event, &candidates[0], u64::MAX),
        ),
    ];
    assert!(context.admissions_for_event(&event, &candidates)[0]);
    for (name, validated) in validations {
        assert!(!validated, "{name}");
        assert!(
            super::authority_lane_allows_apply(
                &event,
                0,
                &candidates,
                &evaluations,
                super::TransitionDecisionPolicy::default(),
                candidates[0].clone(),
                validated,
            )
            .is_none(),
            "{name}"
        );
    }

    let mut tampered = candidates[0].clone();
    tampered.replacement = "другая форма ".to_string();
    let rejected = super::TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        &[tampered],
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
        &context,
    );
    assert_eq!(rejected.selected_index, None);
    assert!(rejected.selected_transition.is_none());
    assert_eq!(candidates[0].replacement, "нужна форма ");

    // The issuer certifies lexical geometry, not permission to edit a protected
    // token. Keep that real capability valid while the actual action verifier
    // rejects the incompatible operator, under a profile needing no extra
    // evidence domains. Frame failure cannot mask this verifier control.
    let (protected_event, protected, protected_context) =
        crate::nanda_wave::l2_field::test_bound_lexical_pipeline(
            "форм",
            "форма",
            TypingErrorClass::ProtectedToken,
        );
    let mut protected_rival = protected.clone();
    protected_rival.replacement = candidates[1].replacement.clone();
    let protected_candidates = [protected, protected_rival];
    let policy = super::TransitionDecisionPolicy {
        l2_phase_apply: false,
        correction_safety: CorrectionSafety::Experimental,
    };
    let mut protected_batch =
        super::TransitionDecisionCore::evaluate_candidates_with_authority_context(
            &protected_event,
            &protected_candidates,
            policy,
            super::DecisionEvidenceMode::FullField(None),
            &protected_context,
        );
    assert!(protected_batch.selected_index.is_none());
    assert!(protected_batch.selected_transition.is_none());
    assert!(protected_batch
        .evaluations
        .iter()
        .all(|evaluation| !evaluation.action.verifier_passed));
    project_advisory_hidden_ambiguity(
        &protected_event,
        &protected_candidates,
        &mut protected_batch.evaluations,
        Some(0),
    );
    let validated = protected_context.admissions_for_event(&protected_event, &protected_candidates);
    assert_eq!(validated, [true, false]);
    assert!(super::authority_lane_allows_apply(
        &protected_event,
        0,
        &protected_candidates,
        &protected_batch.evaluations,
        policy,
        protected_candidates[0].clone(),
        validated[0],
    )
    .is_none());
}

#[test]
fn frame_bound_lexical_ambiguity_deferral_preserves_hidden_certificate_and_rival_vetoes() {
    let (event, candidates, context, mut baseline) = bound_lexical_advisory_fixture();
    project_advisory_hidden_ambiguity(&event, &candidates, &mut baseline, Some(0));
    assert!(context.admissions_for_event(&event, &candidates)[0]);
    for defect in [
        "invalid_certificate",
        "absent_certificate",
        "rejected",
        "witnessed_rival",
        "inconsistent_ambiguity",
    ] {
        let mut evaluations = baseline.clone();
        let signals = &mut evaluations[0].signals;
        match defect {
            "invalid_certificate" => signals.l4_hidden_certificate_valid = false,
            "absent_certificate" => {
                signals.l4_hidden_plan_commitment = 0;
                signals.l4_hidden_certificate_valid = false;
            }
            "rejected" => signals.l4_hidden_disposition = super::L4HiddenDisposition::Rejected,
            "witnessed_rival" | "inconsistent_ambiguity" => {
                signals.l4_hidden_selected_witnessed = true;
                signals.l4_hidden_selected_class = super::predicted_state_id(
                    crate::nanda_wave::phase_field::hash_text(&event.original),
                    baseline[1].action.operator.as_str(),
                    &candidates[1].replacement,
                );
                if defect == "witnessed_rival" {
                    signals.l4_hidden_disposition = super::L4HiddenDisposition::Unobserved;
                }
            }
            _ => unreachable!(),
        }
        let admission = super::authority_lane_allows_apply(
            &event,
            0,
            &candidates,
            &evaluations,
            super::TransitionDecisionPolicy::default(),
            candidates[0].clone(),
            context.admissions_for_event(&event, &candidates)[0],
        );
        assert!(admission.is_none(), "{defect}");
        assert_eq!(candidates[0].replacement, "нужна форма ");
        assert!(evaluations[0].action.verifier_passed);
    }
}

#[test]
fn word_only_rejection_cannot_become_negative_transition_evidence() {
    let event = event("форма нужна форм ");
    let candidates = ["форма нужна форма ", "форма нужна формы "].map(|replacement| {
        let mut candidate = l2_candidate(
            replacement,
            "ProductiveL2V90Surface",
            TypingErrorClass::GrammarAgreement,
        );
        candidate.gate = CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "productive_v90_lattice_requires_common_l3",
        };
        candidate
    });
    let policy = super::TransitionDecisionPolicy::default();
    let baseline = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        policy,
        super::DecisionEvidenceMode::FullField(None),
    );
    assert!(baseline.evaluations[0].signals.l3_pairwise_certified);
    assert!(!baseline.evaluations[1].signals.l3_pairwise_certified);
    assert!(baseline.evaluations[0].action.verifier_passed);
    assert_eq!(baseline.selected_index, Some(0));

    // Hold the real common-L3 result and candidate lattice fixed. Only the
    // independently supplied L4 evidence domain changes in this causal probe.
    for (name, state_specific, attracts, repels, certified, negative, admitted) in [
        ("word_only", false, 0, 0, true, false, true),
        ("exact_negative", true, 0, 1, true, true, false),
        (
            "generic_transition_negative",
            false,
            0,
            16,
            true,
            true,
            false,
        ),
        ("positive_transition", false, 16, 1, true, false, true),
        ("tied_transition", false, 8, 8, true, false, true),
        (
            "numeric_l3_without_certificate",
            false,
            0,
            0,
            false,
            false,
            false,
        ),
    ] {
        let mut evaluations = baseline.evaluations.clone();
        let evaluation = &mut evaluations[0];
        evaluation.signals.l4_signed_milli = -600;
        evaluation.signals.l4_transition_state_specific = state_specific;
        evaluation.signals.l4_transition_attract_count = attracts;
        evaluation.signals.l4_transition_repel_count = repels;
        evaluation.signals.l3_pairwise_certified = certified;
        let signal = evaluation.signals.l4_transition_signal();
        assert_eq!(signal.negative, negative, "{name}: {signal:?}");
        assert_eq!(evaluation.signals.l4_signed_milli, -600, "{name}");
        evaluation.transition.l4_signed_signal = signal;
        let admission = super::authority_lane_allows_apply(
            &event,
            0,
            &candidates,
            &evaluations,
            policy,
            candidates[0].clone(),
            false,
        );
        assert_eq!(admission.is_some(), admitted, "{name}");
        if let Some(admission) = admission {
            assert_eq!(admission.candidate.replacement, "форма нужна форма ");
            assert!(admission.evaluation.action.verifier_passed);
            let receipt = super::DecisionTransitionReceipt::from_selected_candidate(
                &event,
                &admission.candidate,
                &admission.evaluation,
            );
            assert!(receipt
                .projected_transition(&event.original, &admission.candidate.replacement)
                .is_some());
            assert!(receipt
                .projected_transition(&event.original, &candidates[1].replacement)
                .is_none());
        }
    }

    // Owner competition retains the existing generic preference pressure,
    // separately from a hard transition witness. Keep the rival's verified
    // edit and real L3 target; equal ranks exercise this second consumer.
    let mut competitors = candidates.clone();
    competitors[1] = UnifiedCorrectionCandidate::new(
        &candidates[1].replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "missing_letter",
        TypingErrorClass::MissingLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    );
    let competing = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &competitors,
        policy,
        super::DecisionEvidenceMode::FullField(None),
    );
    assert!(competing.evaluations[0].signals.l3_pairwise_certified);
    assert!(competing.evaluations[1].action.verifier_passed);
    assert!(
        !competing.evaluations[1]
            .transition
            .evidence
            .left_context_changed
    );
    for (name, specific, attracts, repels, signed, negative, admitted) in [
        ("word_only_competitor", false, 0, 0, -600, false, true),
        ("preference_boundary", false, 0, 0, -450, false, true),
        ("above_preference_boundary", false, 0, 0, -449, false, false),
        ("exact_negative_competitor", true, 0, 1, 600, true, true),
        (
            "generic_negative_competitor",
            false,
            0,
            16,
            -600,
            true,
            true,
        ),
        ("exact_positive_competitor", true, 16, 1, -600, false, false),
        (
            "generic_positive_competitor",
            false,
            16,
            1,
            -600,
            false,
            true,
        ),
        ("tied_competitor", true, 8, 8, -600, false, true),
    ] {
        let mut evaluations = competing.evaluations.clone();
        let target_rank = evaluations[0].signals.rank_score;
        let rival = &mut evaluations[1];
        rival.signals.rank_score = target_rank;
        rival.signals.l4_signed_milli = signed;
        rival.signals.l4_transition_state_specific = specific;
        rival.signals.l4_transition_attract_count = attracts;
        rival.signals.l4_transition_repel_count = repels;
        rival.transition.l4_signed_signal = rival.signals.l4_transition_signal();
        assert_eq!(
            rival.transition.l4_signed_signal.negative, negative,
            "{name}"
        );
        let admission = super::authority_lane_allows_apply(
            &event,
            0,
            &competitors,
            &evaluations,
            policy,
            competitors[0].clone(),
            false,
        );
        assert_eq!(admission.is_some(), admitted, "{name}");
        if let Some(admission) = admission {
            assert_eq!(admission.candidate.replacement, "форма нужна форма ");
            assert!(admission.evaluation.action.verifier_passed);
            let receipt = super::DecisionTransitionReceipt::from_selected_candidate(
                &event,
                &admission.candidate,
                &admission.evaluation,
            );
            assert!(receipt
                .projected_transition(&event.original, &admission.candidate.replacement)
                .is_some());
            assert!(receipt
                .projected_transition(&event.original, &competitors[1].replacement)
                .is_none());
        }
    }
}

#[test]
fn td112_routed_l3_pair_requires_a_distinct_domain_for_strict() {
    let event = event("форма нужна форм ");
    let recurrent = UnifiedCorrectionCandidate::new(
        "форма нужна форма ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L3Context,
        "PhraseCell32",
        TypingErrorClass::GrammarAgreement,
        CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "td112_l3_pair",
        },
    );
    let alternative = UnifiedCorrectionCandidate::new(
        "форма нужна формы ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L3Context,
        "PhraseCell32",
        TypingErrorClass::GrammarAgreement,
        CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "td112_l3_pair_competitor",
        },
    );

    for (correction_safety, expected_selected) in [
        (CorrectionSafety::Strict, false),
        (CorrectionSafety::Normal, true),
        (CorrectionSafety::Experimental, true),
    ] {
        let candidates = [recurrent.clone(), alternative.clone()];
        let batch = super::TransitionDecisionCore::evaluate_candidates(
            &event,
            &candidates,
            super::TransitionDecisionPolicy {
                l2_phase_apply: false,
                correction_safety,
            },
            super::DecisionEvidenceMode::FullField(None),
        );
        assert!(
            batch.evaluations[0].signals.l3_pairwise_certified,
            "profile={correction_safety:?}: {batch:#?}"
        );
        assert!(!batch.evaluations[1].signals.l3_pairwise_certified);
        assert_eq!(
            batch.selected_index == Some(0),
            expected_selected,
            "profile={correction_safety:?}: {batch:#?}"
        );
        assert_eq!(batch.selected_transition.is_some(), expected_selected);
    }

    let mut corroborated = recurrent;
    corroborated.merge_evidence(UnifiedCorrectionCandidate::new(
        "форма нужна форма ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "td112_unregistered_deterministic_alias",
        TypingErrorClass::GrammarAgreement,
        CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "td112_distinct_lexical_domain",
        },
    ));
    let candidates = [corroborated, alternative];
    let strict = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Strict,
        },
        super::DecisionEvidenceMode::FullField(None),
    );
    assert!(strict.evaluations[0].signals.l3_pairwise_certified);
    assert_eq!(candidates[0].evidence_count(), 2);
    assert_eq!(strict.selected_index, Some(0), "{strict:#?}");
    assert!(strict.selected_transition.is_some());
}

#[test]
fn td112_decision_core_structural_veto_dominates_profile_admission() {
    let event = event("содержкой ");
    let candidate = UnifiedCorrectionCandidate::new(
        "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        "L2SurfaceMotifCell32",
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "td112_structural_veto",
        },
    );
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &[candidate],
        super::TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        super::DecisionEvidenceMode::FullField(None),
    );

    assert_eq!(batch.evaluations.len(), 1);
    assert!(batch.evaluations[0].action.left_context_changed);
    assert!(!batch.evaluations[0].action.verifier_passed);
    assert!(batch.selected_index.is_none(), "{batch:#?}");
    assert!(batch.selected_transition.is_none());
}

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
fn suggest_only_verified_tail_boundary_can_enter_authority_evaluation() {
    let event = event("я думаю допусти мнабираю ");
    let candidate = UnifiedCorrectionCandidate::new(
        "я думаю допустим набираю ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Boundary,
        "moved_prefix_pair",
        TypingErrorClass::BoundaryShift,
        CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "edit_transition_not_verified",
        },
    );

    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &[candidate],
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
    );

    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
}

#[test]
fn td113_verified_boundary_merge_is_not_reinterpreted_as_lexical_drift() {
    let event = event("текст е ");
    let candidate = UnifiedCorrectionCandidate::new(
        "тексте ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Boundary,
        "verified_boundary_merge_contract",
        TypingErrorClass::SplitWord,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "boundary_transition_verified",
        },
    );

    let action = crate::typing_transition::action::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    );
    assert!(action.verifier_passed, "{action:?}");
    assert_eq!(
        action.edit_operator,
        crate::text_edit::TransitionOperator::BoundaryMergeSplit
    );

    let admission = admit(&event, &candidate, 2, CorrectionSourceRole::Boundary, false);
    assert!(admission.allow_apply, "{admission:?}");
}

#[test]
fn td113_unverified_boundary_transition_remains_blocked() {
    let event = event("текст е ");
    let candidate = UnifiedCorrectionCandidate::new(
        "чужой тексте ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Boundary,
        "unverified_boundary_contract",
        TypingErrorClass::SplitWord,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "boundary_transition_claim",
        },
    );

    let action = crate::typing_transition::action::verify_action_operator(
        &event.original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    );
    assert!(!action.verifier_passed, "{action:?}");

    let admission = admit(&event, &candidate, 2, CorrectionSourceRole::Boundary, false);
    assert!(!admission.allow_apply, "{admission:?}");
}

#[test]
fn td113_zero_loss_verified_boundary_competitor_blocks_close_lossy_candidate() {
    let event = event("т ыпочитай ");
    let lossy = UnifiedCorrectionCandidate::new(
        "т почитай ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "lossy_typo_contract",
        TypingErrorClass::ExtraLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    );
    let structural = UnifiedCorrectionCandidate::new(
        "ты почитай ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Boundary,
        "zero_loss_boundary_contract",
        TypingErrorClass::BoundaryShift,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "boundary_transition_verified",
        },
    );

    let candidates = [lossy, structural];
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        super::DecisionEvidenceMode::FullField(None),
    );

    assert!(batch.evaluations[1].action.verifier_passed, "{batch:#?}");
    assert_eq!(
        batch.evaluations[1].action.edit_operator,
        crate::text_edit::TransitionOperator::BoundaryShift
    );
    assert_eq!(batch.evaluations[1].explanation.lost_mass_milli, 0);
    assert!(batch.evaluations[0].explanation.lost_mass_milli > 0);
    assert_eq!(batch.selected_index, Some(1), "{batch:#?}");
}

#[test]
fn td113_exact_function_word_boundary_split_matrix() {
    for (original, replacement) in [
        ("Какие документыим ", "Какие документы им "),
        ("Какие словаим ", "Какие слова им "),
        ("Какие документыдля ", "Какие документы для "),
        ("Какие вдокументы ", "Какие в документы "),
    ] {
        assert!(
            super::admission::exact_current_token_function_word_split(original, replacement),
            "expected exact function-word split: {original:?} -> {replacement:?}"
        );
    }

    for (original, replacement) in [
        ("самка схема парочинная ", "самка схема паро чинная "),
        ("Какие документаим ", "Какие документы им "),
        ("вместе ", "в месте "),
    ] {
        assert!(
            !super::admission::exact_current_token_function_word_split(original, replacement),
            "unexpected exact function-word split: {original:?} -> {replacement:?}"
        );
    }
}

#[test]
fn td113_ambiguous_function_word_split_has_no_apply_authority() {
    for correction_safety in [CorrectionSafety::Normal, CorrectionSafety::Experimental] {
        let event = event("воротаим ");
        let boundary = UnifiedCorrectionCandidate::new(
            "ворота им ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "ambiguous_function_word_boundary",
            TypingErrorClass::GluedWords,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "boundary_transition_verified",
            },
        );
        let candidates = [boundary];
        let batch = super::TransitionDecisionCore::evaluate_candidates(
            &event,
            &candidates,
            super::TransitionDecisionPolicy {
                l2_phase_apply: false,
                correction_safety,
            },
            super::DecisionEvidenceMode::FullField(None),
        );

        assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
        assert_eq!(
            batch.evaluations[0].action.edit_operator,
            crate::text_edit::TransitionOperator::BoundaryMergeSplit
        );
        assert!(
            super::admission::exact_current_token_function_word_split(
                &event.original,
                &candidates[0].replacement,
            ),
            "the surface is a real function-word split; ambiguity must be handled by authority"
        );
        assert!(
            crate::russian_typo_candidates::any_single_damerau_edit_candidate(
                event.current_word.trim(),
                crate::russian_lexicon::has_clean_russian_surface_certificate,
            ),
            "the complete frontier must expose the clean one-word competitor"
        );
        assert_eq!(
            batch.selected_index, None,
            "surface-lexical split evidence must not auto-apply over a clean one-word competitor under {correction_safety:?}: {batch:#?}"
        );
    }
}

#[test]
fn td113_grounded_content_split_with_clean_competitor_has_no_apply_authority() {
    for correction_safety in [CorrectionSafety::Normal, CorrectionSafety::Experimental] {
        let event = event("авторручка ");
        let boundary = UnifiedCorrectionCandidate::new(
            "автор ручка ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "content_content_boundary",
            TypingErrorClass::GluedWords,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "boundary_transition_verified",
            },
        )
        .with_l2_boundary_target_grounding();
        let candidates = [boundary];
        let batch = super::TransitionDecisionCore::evaluate_candidates(
            &event,
            &candidates,
            super::TransitionDecisionPolicy {
                l2_phase_apply: false,
                correction_safety,
            },
            super::DecisionEvidenceMode::FullField(None),
        );

        assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
        assert_eq!(
            batch.evaluations[0].action.edit_operator,
            crate::text_edit::TransitionOperator::BoundaryMergeSplit
        );
        assert!(crate::text_metrics::current_token_boundary_split(
            &event.original,
            &candidates[0].replacement,
        ));
        assert!(candidates[0].has_l2_boundary_target_grounding());
        assert!(crate::nanda_wave::l2::ime_l2_boundary_target_evidence(
            "авторручка",
            "автор ручка"
        ));
        assert!(
            crate::russian_typo_candidates::has_clean_single_damerau_edit_candidate("авторручка"),
            "the complete one-edit frontier must expose the competing clean word"
        );
        assert!(
            !super::admission::exact_current_token_function_word_split(
                &event.original,
                &candidates[0].replacement,
            ),
            "content-content geometry must not become function-word authority"
        );
        assert_eq!(batch.selected_index, None, "{batch:#?}");
    }
}

#[test]
fn td113_standalone_repaired_boundary_split_has_no_apply_authority() {
    for correction_safety in [CorrectionSafety::Normal, CorrectionSafety::Experimental] {
        let event = event("Какие документаим ");
        let boundary = UnifiedCorrectionCandidate::new(
            "Какие документы им ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "repaired_boundary_contract",
            TypingErrorClass::GluedWords,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "boundary_transition_verified",
            },
        );
        let candidates = [boundary];
        let batch = super::TransitionDecisionCore::evaluate_candidates(
            &event,
            &candidates,
            super::TransitionDecisionPolicy {
                l2_phase_apply: false,
                correction_safety,
            },
            super::DecisionEvidenceMode::FullField(None),
        );

        assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
        assert_eq!(
            batch.evaluations[0].action.edit_operator,
            crate::text_edit::TransitionOperator::BoundaryMergeSplit
        );
        assert!(crate::text_metrics::current_token_repaired_boundary_split(
            &event.original,
            &candidates[0].replacement,
        ));
        assert!(!candidates[0].has_l2_boundary_target_grounding());
        assert!(
            !super::admission::exact_current_token_function_word_split(
                &event.original,
                &candidates[0].replacement,
            ),
            "repaired split geometry must not become semantic authority"
        );
        assert_eq!(batch.selected_index, None, "{batch:#?}");
    }
}

#[test]
fn td113_exact_current_token_boundary_split_dominates_close_single_token_repair() {
    let (original, boundary_replacement, repair_replacement) = (
        "Какие документыим ",
        "Какие документы им ",
        "Какие документыми ",
    );
    let event = event(original);
    let boundary = UnifiedCorrectionCandidate::new(
        boundary_replacement,
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::Boundary,
        "canonical_boundary_contract",
        TypingErrorClass::GluedWords,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "boundary_transition_verified",
        },
    );
    let transposition = UnifiedCorrectionCandidate::new(
        repair_replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "adjacent_transposition_contract",
        TypingErrorClass::AdjacentTransposition,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    );

    let candidates = [boundary, transposition];
    assert!(
        !crate::russian_lexicon::has_clean_russian_surface_certificate("документыми"),
        "a morphology-shaped but unattested target must not block the verified split"
    );
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        super::DecisionEvidenceMode::FullField(None),
    );

    assert_eq!(batch.evaluations.len(), 2, "{batch:#?}");
    assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
    assert!(batch.evaluations[1].action.verifier_passed, "{batch:#?}");
    assert!(crate::text_metrics::current_token_boundary_split(
        &event.original,
        &candidates[0].replacement,
    ));
    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
}

#[test]
fn td113_exact_content_content_boundary_split_remains_blocked() {
    let event = event("самка схема парочинная ");
    let boundary = UnifiedCorrectionCandidate::new(
        "самка схема паро чинная ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::Boundary,
        "content_boundary_contract",
        TypingErrorClass::GluedWords,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "boundary_transition_verified",
        },
    );
    let close_repair = UnifiedCorrectionCandidate::new(
        "самка схема перочинная ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        "surface_repair_contract",
        TypingErrorClass::LetterSubstitution,
        CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_current_word_surface_drift",
        },
    );

    let candidates = [boundary, close_repair];
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        super::DecisionEvidenceMode::FullField(None),
    );

    assert_eq!(batch.evaluations.len(), 2, "{batch:#?}");
    assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
    assert!(batch.evaluations[1].action.verifier_passed, "{batch:#?}");
    assert!(crate::text_metrics::current_token_boundary_split(
        &event.original,
        &candidates[0].replacement,
    ));
    let mut boundary_evaluation = batch.evaluations[0].clone();
    boundary_evaluation.action.changed_tokens = 2;
    boundary_evaluation.explanation.lost_mass_milli = 0;
    boundary_evaluation.explanation.preservation_milli = 1_000;
    boundary_evaluation.explanation.operator_fit_milli = 1_000;
    let mut competitor_evaluation = batch.evaluations[1].clone();
    competitor_evaluation.explanation.lost_mass_milli = 100;
    competitor_evaluation.explanation.preservation_milli = 900;
    competitor_evaluation.explanation.operator_fit_milli = 900;
    assert!(
        !super::admission::verified_zero_loss_boundary_dominates_for_test(
            &boundary_evaluation,
            &competitor_evaluation,
        ),
        "content-content BoundaryMergeSplit must not enter the generic zero-loss dominance path even under its exact bypass metric tuple: {batch:#?}"
    );
    assert_eq!(batch.selected_index, None, "{batch:#?}");
}

#[test]
fn td113_repaired_boundary_split_does_not_gain_exact_dominance_exception() {
    let event = event("Какие документаим ");
    let repaired_boundary = UnifiedCorrectionCandidate::new(
        "Какие документы им ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::Boundary,
        "repaired_boundary_contract",
        TypingErrorClass::GluedWords,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "boundary_transition_verified",
        },
    );
    let known_word_repair = UnifiedCorrectionCandidate::new(
        "Какие документами ",
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "known_word_repair_contract",
        TypingErrorClass::AdjacentTransposition,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    );

    let candidates = [repaired_boundary, known_word_repair];
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        super::TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        super::DecisionEvidenceMode::FullField(None),
    );

    assert_eq!(batch.evaluations.len(), 2, "{batch:#?}");
    assert!(batch.evaluations[0].action.verifier_passed, "{batch:#?}");
    assert!(batch.evaluations[1].action.verifier_passed, "{batch:#?}");
    assert!(!crate::text_metrics::current_token_boundary_split(
        &event.original,
        &candidates[0].replacement,
    ));
    assert!(crate::text_metrics::current_token_repaired_boundary_split(
        &event.original,
        &candidates[0].replacement,
    ));
    assert_ne!(batch.selected_index, Some(0), "{batch:#?}");
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

fn layout_candidate(replacement: &str) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Layout,
        "contextual_layout_en_to_ru",
        TypingErrorClass::WrongLayout,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "layout_projection_verified",
        },
    )
}

fn layout_then_typo_candidate(replacement: &str) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::LayoutThenTypo,
        "layout_then_known_word",
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "layout_then_typo_verified",
        },
    )
}

fn l2_candidate(
    replacement: &str,
    source_id: &str,
    error_class: TypingErrorClass,
) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        source_id,
        error_class,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    )
}

#[test]
fn td117_alias_authority_reuses_one_surface_evaluation_for_every_settlement_stage() {
    let event = event("Еленапросит ");
    let mut canonical = UnifiedCorrectionCandidate::new(
        "Елена просит ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        crate::nanda_wave::l2_field::CANONICAL_L2_SURFACE_SOURCE_ID,
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "td117_canonical_surface",
        },
    );
    canonical.merge_evidence(
        UnifiedCorrectionCandidate::new(
            "Елена просит ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "td117_boundary_owner",
            TypingErrorClass::GluedWords,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "td117_boundary_owner",
            },
        )
        .with_l2_boundary_target_grounding(),
    );

    let before = super::td117_surface_stage_settlements();
    let batch = super::TransitionDecisionCore::evaluate_candidates(
        &event,
        &[canonical],
        super::TransitionDecisionPolicy::default(),
        super::DecisionEvidenceMode::FullField(None),
    );
    let after = super::td117_surface_stage_settlements();

    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
    assert_eq!(after.morphology.saturating_sub(before.morphology), 1);
    assert_eq!(after.interference.saturating_sub(before.interference), 1);
    assert_eq!(after.l4_hidden.saturating_sub(before.l4_hidden), 1);
}

#[test]
fn td117_alias_lanes_cannot_bypass_surface_keep_or_veto() {
    let event = event("Еленапросит ");
    let mut merged = UnifiedCorrectionCandidate::new(
        "Елена просит ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        crate::nanda_wave::l2_field::CANONICAL_L2_SURFACE_SOURCE_ID,
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "td117_canonical_surface",
        },
    );
    merged.merge_evidence(
        UnifiedCorrectionCandidate::new(
            "Елена просит ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "td117_boundary_owner",
            TypingErrorClass::GluedWords,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "td117_boundary_owner",
            },
        )
        .with_l2_boundary_target_grounding(),
    );

    for action in [CandidateGateAction::KeepOriginal, CandidateGateAction::Veto] {
        let mut blocked = merged.clone();
        blocked.gate = CandidateGateDecision {
            action,
            reason: "td117_surface_blocker",
        };
        let batch = super::TransitionDecisionCore::evaluate_candidates(
            &event,
            &[blocked],
            super::TransitionDecisionPolicy::default(),
            super::DecisionEvidenceMode::FullField(None),
        );

        assert_eq!(batch.selected_index, None, "action={action:?}: {batch:#?}");
        assert!(batch.selected_candidate.is_none());
        assert!(batch.selected_transition.is_none());
    }
}

fn admit(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    context_state_support: bool,
) -> super::TransitionAdmission {
    admit_with_l4_signal(
        event,
        candidate,
        candidate_count,
        source_role,
        context_state_support,
        false,
        crate::typing_transition::L4SignedTransitionSignal {
            negative: false,
            state_specific: false,
            attract_count: 0,
            repel_count: 0,
        },
    )
}

fn admit_with_l4_signal(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    candidate_count: usize,
    source_role: CorrectionSourceRole,
    context_state_support: bool,
    operator_consensus_witness: bool,
    l4_signed_signal: crate::typing_transition::L4SignedTransitionSignal,
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
            l4_signed_signal,
        },
    );
    admit_evaluated_hidden_transition(
        candidate_count,
        source_role,
        context_state_support,
        operator_consensus_witness,
        context_state_support
            || transition.l4_signed_signal.exact_positive()
            || super::admission::exact_current_token_function_word_split(
                &event.original,
                &candidate.replacement,
            ),
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
fn hidden_state_blocks_live_known_form_drifts_from_logs() {
    for (input, replacement, error_class) in [
        ("новости ", "новость ", TypingErrorClass::LetterSubstitution),
        ("модели ", "модель ", TypingErrorClass::LetterSubstitution),
        ("коде ", "код ", TypingErrorClass::ExtraLetter),
        ("вышли ", "вышил ", TypingErrorClass::AdjacentTransposition),
        (
            "окнах ",
            "локонах ",
            TypingErrorClass::SparseInternalMultiOmission,
        ),
    ] {
        let admission = admit(
            &event(input),
            &l2_candidate(replacement, "CanonicalL2FieldSurface", error_class),
            2,
            CorrectionSourceRole::L2Surface,
            false,
        );

        assert!(!admission.allow_apply, "{input:?} -> {replacement:?}");
        assert!(
            matches!(
                admission.reason,
                "known_form_drift_needs_state_proof" | "latent_known_word_drift_needs_state_proof"
            ),
            "{input:?} -> {replacement:?}"
        );
    }
}

#[test]
fn hidden_state_blocks_short_transposition_fragments_from_logs() {
    for (input, replacement) in [
        ("ая ", "яа "),
        ("ту ", "ут "),
        ("вно ", "вон "),
        ("ям ", "мя "),
    ] {
        let admission = admit(
            &event(input),
            &l2_candidate(
                replacement,
                "CanonicalL2FieldSurface",
                TypingErrorClass::AdjacentTransposition,
            ),
            2,
            CorrectionSourceRole::L2Surface,
            false,
        );

        assert!(!admission.allow_apply, "{input:?} -> {replacement:?}");
        assert_eq!(
            admission.reason, "short_transposition_needs_state_proof",
            "{input:?} -> {replacement:?}"
        );
    }
}

#[test]
fn exact_l4_state_proof_allows_known_form_drift() {
    let admission = admit_with_l4_signal(
        &event("новости "),
        &l2_candidate(
            "новость ",
            "CanonicalL2FieldSurface",
            TypingErrorClass::LetterSubstitution,
        ),
        2,
        CorrectionSourceRole::L2Surface,
        false,
        false,
        crate::typing_transition::L4SignedTransitionSignal {
            negative: false,
            state_specific: true,
            attract_count: 2,
            repel_count: 0,
        },
    );

    assert!(admission.allow_apply, "{admission:?}");
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
    assert!(super::known_word_drift_has_authority(false, true));
}

#[test]
fn l2_operator_phase_is_not_context_state_proof() {
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

#[test]
fn admission_truth_table_uses_verifier_latent_invariants_and_signed_l4_memory() {
    struct Case {
        name: &'static str,
        event: TypingErrorEvent,
        candidate: UnifiedCorrectionCandidate,
        source_role: CorrectionSourceRole,
        strong_transition_support: bool,
        operator_consensus_witness: bool,
        l4_signed_signal: crate::typing_transition::L4SignedTransitionSignal,
        expected_reason: Option<&'static str>,
    }

    let neutral_l4 = crate::typing_transition::L4SignedTransitionSignal {
        negative: false,
        state_specific: false,
        attract_count: 0,
        repel_count: 0,
    };
    let negative_l4 = crate::typing_transition::L4SignedTransitionSignal {
        negative: true,
        state_specific: true,
        attract_count: 0,
        repel_count: 1,
    };
    let generic_negative_l4 = crate::typing_transition::L4SignedTransitionSignal {
        negative: true,
        state_specific: false,
        attract_count: 0,
        repel_count: 16,
    };
    let cases = [
        Case {
            name: "verified_current_word",
            event: event("провека "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: false,
            l4_signed_signal: neutral_l4,
            expected_reason: None,
        },
        Case {
            name: "unverified_context_change",
            event: event("можем "),
            candidate: candidate("мы модем ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: false,
            l4_signed_signal: neutral_l4,
            expected_reason: Some("latent_context_unverified"),
        },
        Case {
            name: "known_word_drift_without_state_proof",
            event: event("мы можем "),
            candidate: candidate("мы модем ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: neutral_l4,
            expected_reason: Some("latent_known_word_drift_needs_state_proof"),
        },
        Case {
            name: "learned_l4_signed_negative",
            event: event("провека "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: true,
            l4_signed_signal: negative_l4,
            expected_reason: Some("latent_l4_negative_transition_memory"),
        },
        Case {
            name: "generic_l4_negative_without_consensus",
            event: event("преоверка "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: false,
            l4_signed_signal: generic_negative_l4,
            expected_reason: Some("latent_l4_negative_transition_memory"),
        },
        Case {
            name: "operator_consensus_survives_generic_l4_negative",
            event: event("преоверка "),
            candidate: candidate("проверка ", "composite_ru_typo"),
            source_role: CorrectionSourceRole::DeterministicTypo,
            strong_transition_support: true,
            operator_consensus_witness: true,
            l4_signed_signal: generic_negative_l4,
            expected_reason: None,
        },
        Case {
            name: "verified_layout_survives_generic_l4_negative",
            event: event("gjhn "),
            candidate: layout_candidate("порт "),
            source_role: CorrectionSourceRole::Layout,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: generic_negative_l4,
            expected_reason: None,
        },
        Case {
            name: "verified_layout_then_typo_survives_generic_l4_negative",
            event: event("lfkmit "),
            candidate: layout_then_typo_candidate("дальше "),
            source_role: CorrectionSourceRole::Layout,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: generic_negative_l4,
            expected_reason: None,
        },
        Case {
            name: "verified_layout_blocked_by_state_specific_l4_negative",
            event: event("gjhn "),
            candidate: layout_candidate("порт "),
            source_role: CorrectionSourceRole::Layout,
            strong_transition_support: false,
            operator_consensus_witness: false,
            l4_signed_signal: negative_l4,
            expected_reason: Some("latent_l4_negative_transition_memory"),
        },
    ];

    for case in cases {
        let admission = admit_with_l4_signal(
            &case.event,
            &case.candidate,
            1,
            case.source_role,
            case.strong_transition_support,
            case.operator_consensus_witness,
            case.l4_signed_signal,
        );

        match case.expected_reason {
            Some(reason) => {
                assert!(
                    !admission.allow_apply,
                    "{} unexpectedly admitted: {admission:?}",
                    case.name
                );
                assert_eq!(admission.reason, reason, "{}", case.name);
            }
            None => assert!(
                admission.allow_apply,
                "{} unexpectedly rejected: {admission:?}",
                case.name
            ),
        }
    }
}

#[test]
fn exact_positive_l4_memory_can_promote_a_suggestion_to_authority_evaluation() {
    let neutral = crate::typing_transition::L4SignedTransitionSignal {
        negative: false,
        state_specific: false,
        attract_count: 0,
        repel_count: 0,
    };
    let exact_positive = crate::typing_transition::L4SignedTransitionSignal {
        negative: false,
        state_specific: true,
        attract_count: 6,
        repel_count: 0,
    };
    let exact_negative = crate::typing_transition::L4SignedTransitionSignal {
        negative: true,
        state_specific: true,
        attract_count: 0,
        repel_count: 8,
    };

    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::Eligible,
        false,
        neutral,
    ));
    assert!(!producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        false,
        neutral,
    ));
    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        false,
        exact_positive,
    ));
    assert!(!producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        false,
        exact_negative,
    ));
    assert!(producer_allows_authority_evaluation(
        CandidateGateAction::SuggestOnly,
        true,
        neutral,
    ));
}

#[test]
fn exact_positive_l4_memory_outvotes_only_unresolved_competitors() {
    assert!(unresolved_competitor_blocks(false, true));
    assert!(!unresolved_competitor_blocks(true, true));
    assert!(!unresolved_competitor_blocks(false, false));
}
