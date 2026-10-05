use super::*;
use crate::correction_core::CorrectionDecisionSource;

fn identity(lemma: u32, labels: &str) -> MorphologySlotIdentity {
    MorphologySlotIdentity {
        domain: MorphologySlotIdentityDomain::CanonicalFeature,
        lemma_id: lemma,
        slot_id: crate::nanda_wave::l2_field::parse_canonical_morphology_features(labels).unwrap(),
    }
}

fn normalized(lemma: u32, labels: &str) -> Vec<Reading> {
    readings(vec![identity(lemma, labels)]).unwrap()
}

fn option(index: usize, score: f32, labels: &str) -> OptionReading {
    OptionReading {
        index,
        score,
        readings: normalized(7, labels),
    }
}

fn partner(person: u8, number: u8) -> BTreeSet<Signature> {
    BTreeSet::from([Signature::PersonNumber(person, number)])
}

#[test]
fn completed101_selects_unique_native_top_among_multiple_supported_surfaces() {
    let source = normalized(7, "verb:pres:ind:p3:sg:imperf");
    let options = [
        option(0, 9.0, "verb:pres:ind:p1:sg:imperf"),
        option(1, 3.0, "verb:pres:ind:p3:pl:imperf"),
        option(2, 4.0, "verb:fut:ind:p3:pl:perf"),
    ];
    assert_eq!(
        choose(&source, &options, &[partner(3, 2)], &[]),
        (Some(2), "agreement_compatible_signature_preferred")
    );
    // S is a selection among retained surfaces; no tense/gold correctness is
    // inferred from the shared reduced agreement signature.
    assert_eq!(
        options.iter().map(|o| o.score).collect::<Vec<_>>(),
        vec![9.0, 3.0, 4.0]
    );
}

#[test]
fn old_any_partner_veto_and_literal_personal_exception_are_preserved() {
    let source = normalized(7, "verb:pres:ind:p3:sg");
    let options = [
        option(0, 9.0, "verb:pres:ind:p3:sg"),
        option(1, 2.0, "verb:pres:ind:p3:pl"),
    ];
    assert_eq!(
        choose(&source, &options, &[partner(3, 2), partner(3, 1)], &[]).1,
        "agreement_old_top_has_partner_support"
    );
    let personal = [
        option(0, 9.0, "verb:pres:ind:p3:sg"),
        option(1, 2.0, "verb:pres:ind:p2:pl"),
    ];
    assert_eq!(
        choose(&source, &personal, &[partner(2, 2), partner(3, 1)], &[]).0,
        Some(1)
    );
    let competing_personal = [
        option(0, 9.0, "verb:pres:ind:p1:sg"),
        option(1, 2.0, "verb:pres:ind:p2:pl"),
    ];
    assert_eq!(
        choose(
            &source,
            &competing_personal,
            &[partner(2, 2), partner(1, 1)],
            &[]
        )
        .0,
        None
    );
}

#[test]
fn ties_nearest_and_candidate_ambiguity_do_not_relax_legacy_c() {
    let source = normalized(7, "verb:pres:ind:p3:sg");
    let tied_old = [
        option(0, 4.0, "verb:pres:ind:p1:sg"),
        option(1, 4.0, "verb:pres:ind:p3:pl"),
    ];
    assert_eq!(
        choose(&source, &tied_old, &[partner(3, 2)], &[]).1,
        "agreement_tied_old_top"
    );
    let tied_support = [
        option(0, 9.0, "verb:pres:ind:p1:sg"),
        option(1, 4.0, "verb:pres:ind:p3:pl"),
        option(2, 4.0, "verb:fut:ind:p3:pl"),
    ];
    assert_eq!(
        choose(&source, &tied_support, &[partner(3, 2)], &[]).1,
        "agreement_tied_supported_top"
    );
    let nearest = BTreeSet::from([Signature::PersonNumber(3, 1), Signature::PersonNumber(3, 2)]);
    assert_eq!(
        choose(&source, &tied_support, &[nearest, partner(3, 2)], &[]).0,
        None
    );
    let mut ambiguous = option(1, 4.0, "verb:pres:ind:p3:pl");
    ambiguous
        .readings
        .extend(normalized(7, "verb:pres:ind:p2:pl"));
    assert_eq!(
        choose(
            &source,
            &[option(0, 9.0, "verb:pres:ind:p1:sg"), ambiguous],
            &[partner(3, 2)],
            &[]
        )
        .0,
        None
    );
}

#[test]
fn full_mask_domains_family_identity_and_missing_person_remain_distinct() {
    let mut productive = identity(7, "verb:pres:ind:p3:pl");
    productive.domain = MorphologySlotIdentityDomain::ProductiveV1;
    assert!(readings(vec![productive]).is_none());
    let mut invalid = identity(7, "verb:pres:ind:p3:pl");
    invalid.slot_id = 0;
    assert!(readings(vec![invalid]).is_none());
    let past = normalized(7, "verb:past:ind:sg:fem");
    assert!(past[0].verb.is_none());
    let source = normalized(7, "verb:pres:ind:p3:sg");
    let mut other_family = option(1, 4.0, "verb:pres:ind:p3:pl");
    other_family.readings = normalized(8, "verb:pres:ind:p3:pl");
    assert_eq!(
        choose(
            &source,
            &[option(0, 9.0, "verb:pres:ind:p1:sg"), other_family],
            &[partner(3, 2)],
            &[]
        )
        .0,
        None
    );
    assert!(readings(vec![identity(7, "verb:pres:ind:p3:pl"); MAX_READINGS + 1]).is_none());
}

#[test]
fn signature_cardinality_is_not_full_reading_cardinality() {
    let mut source = normalized(7, "verb:pres:ind:p3:sg:imperf");
    source.extend(normalized(7, "verb:fut:ind:p2:sg:perf"));
    let options = [
        option(0, 9.0, "verb:pres:ind:p1:sg"),
        option(1, 4.0, "verb:pres:ind:p3:pl"),
    ];
    assert_eq!(choose(&source, &options, &[partner(3, 2)], &[]).0, Some(1));
    source.extend(normalized(7, "adj:nom:sg:masc"));
    assert_eq!(
        choose(&source, &options, &[partner(3, 2)], &[]).1,
        "agreement_unknown_source_role"
    );
    let mut same_signature = option(1, 4.0, "verb:pres:ind:p3:pl");
    same_signature
        .readings
        .extend(normalized(7, "verb:fut:ind:p3:pl"));
    assert_eq!(
        choose(
            &normalized(7, "verb:pres:ind:p3:sg"),
            &[options[0].clone_for_test(), same_signature],
            &[partner(3, 2)],
            &[]
        )
        .0,
        Some(1)
    );
}

impl OptionReading {
    fn clone_for_test(&self) -> Self {
        Self {
            index: self.index,
            score: self.score,
            readings: self.readings.clone(),
        }
    }
}

fn fixture(
    text: &str,
    replacements: &[&str],
    scores: &[f32],
) -> (
    TypingErrorEvent,
    Vec<UnifiedCorrectionCandidate>,
    Vec<CandidateDecisionEvaluation>,
    Vec<Option<AuthorityLaneAdmission>>,
) {
    let event = TypingErrorEvent {
        original: text.to_owned(),
        core: text.trim().to_owned(),
        current_word: text.split_whitespace().last().unwrap().to_owned(),
        input_class: TypingErrorClass::GrammarAgreement,
    };
    let candidates = replacements
        .iter()
        .map(|replacement| {
            UnifiedCorrectionCandidate::new(
                *replacement,
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::L2Surface,
                "agreement-contract-fixture",
                TypingErrorClass::GrammarAgreement,
                CandidateGateDecision {
                    action: CandidateGateAction::Eligible,
                    reason: "class_allows_apply",
                },
            )
        })
        .collect::<Vec<_>>();
    // Use actual core evaluations, action/verifier and transition types. Only
    // the frozen numeric preference is controlled for this ordering boundary.
    let mut evaluations = TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        TransitionDecisionPolicy::default(),
        DecisionEvidenceMode::FullField(None),
    )
    .evaluations;
    for (evaluation, score) in evaluations.iter_mut().zip(scores) {
        evaluation.signals.rank_score = *score;
    }
    let admissions = candidates
        .iter()
        .zip(&evaluations)
        .map(|(candidate, evaluation)| {
            Some(AuthorityLaneAdmission {
                candidate: candidate.clone(),
                evaluation: evaluation.clone(),
            })
        })
        .collect();
    (event, candidates, evaluations, admissions)
}

fn lookup(surface: &str) -> Option<Vec<MorphologySlotIdentity>> {
    let (lemma, features) = match surface {
        "пишет" => (7, "verb:pres:ind:p3:sg"),
        "пишу" => (7, "verb:pres:ind:p1:sg"),
        "пишут" => (7, "verb:pres:ind:p3:pl"),
        "они" => (8, "pron:nom:p3:pl"),
        "я" => (9, "pron:nom:p1:sg"),
        _ => return Some(Vec::new()),
    };
    Some(vec![identity(lemma, features)])
}

#[test]
fn admitted_native_adapter_reorders_without_mutating_candidates_evaluations_or_receipts() {
    let (event, candidates, evaluations, admissions) =
        fixture("они пишет ", &["они пишу ", "они пишут "], &[9.0, 2.0]);
    let frozen_candidates = candidates.clone();
    let frozen_evaluations = format!("{evaluations:?}");
    let frozen_admissions = admissions
        .iter()
        .map(|a| {
            a.as_ref()
                .map(|a| (a.candidate.clone(), format!("{:?}", a.evaluation)))
        })
        .collect::<Vec<_>>();
    let selected = select_with_readings(&event, &candidates, &evaluations, &admissions, lookup);
    assert_eq!(selected.selected, Some(1));
    let admission = admissions[1].as_ref().unwrap();
    assert!(admission.evaluation.action.verifier_passed);
    assert!(!admission.evaluation.action.left_context_changed);
    assert_eq!(admission.evaluation.action.changed_tokens, 1);
    let receipt = DecisionTransitionReceipt::from_selected_candidate(
        &event,
        &admission.candidate,
        &admission.evaluation,
    );
    assert!(receipt
        .projected_transition(&event.original, &candidates[1].replacement)
        .is_some());
    assert_eq!(candidates, frozen_candidates);
    assert_eq!(format!("{evaluations:?}"), frozen_evaluations);
    assert_eq!(
        admissions
            .iter()
            .map(|a| a
                .as_ref()
                .map(|a| (a.candidate.clone(), format!("{:?}", a.evaluation))))
            .collect::<Vec<_>>(),
        frozen_admissions
    );
}

#[test]
fn only_existing_admissions_participate_and_no_edit_never_gains_authority() {
    let (event, candidates, evaluations, mut admissions) =
        fixture("они пишет ", &["они пишу ", "они пишут "], &[9.0, 2.0]);
    admissions[1] = None;
    let mut calls = 0;
    let result = select_with_readings(&event, &candidates, &evaluations, &admissions, |_| {
        calls += 1;
        panic!("one admission needs no evidence")
    });
    assert_eq!(result.selected, Some(0));
    assert_eq!(calls, 0);
    admissions[0] = None;
    assert_eq!(
        select_with_readings(&event, &candidates, &evaluations, &admissions, |_| panic!(
            "NoEdit must remain NoEdit"
        ))
        .selected,
        None
    );
    // Actual producer gates are still applied in the connected core before
    // this boundary, rather than inferred from a grammatical match.
    let mut denied = candidates.clone();
    denied[0].gate.action = CandidateGateAction::KeepOriginal;
    denied[1].gate.action = CandidateGateAction::Veto;
    let batch = TransitionDecisionCore::evaluate_candidates(
        &event,
        &denied,
        TransitionDecisionPolicy::default(),
        DecisionEvidenceMode::FullField(None),
    );
    assert!(batch.selected_index.is_none());
    assert!(batch.selected_candidate.is_none());
    assert!(batch.selected_transition.is_none());
    assert_eq!(batch.evaluations.len(), denied.len());
}

#[test]
fn unknown_provider_nonfinite_and_score_ties_preserve_original_decision_order() {
    let (event, candidates, mut evaluations, admissions) =
        fixture("они пишет ", &["они пишу ", "они пишут "], &[9.0, 2.0]);
    assert_eq!(
        select_with_readings(&event, &candidates, &evaluations, &admissions, |_| None).selected,
        Some(0)
    );
    evaluations[1].signals.rank_score = f32::NAN;
    let baseline =
        (0..2).max_by(|a, b| compare_candidate_decision_order(*a, *b, &candidates, &evaluations));
    assert_eq!(
        select_with_readings(&event, &candidates, &evaluations, &admissions, lookup).selected,
        baseline
    );
    evaluations[1].signals.rank_score = evaluations[0].signals.rank_score;
    let baseline =
        (0..2).max_by(|a, b| compare_candidate_decision_order(*a, *b, &candidates, &evaluations));
    assert_eq!(
        select_with_readings(&event, &candidates, &evaluations, &admissions, lookup).selected,
        baseline
    );
}

#[test]
fn actual_last_word_punctuation_window_and_left_context_replacements_are_preserved() {
    for text in ["они, пишет ", "они — пишет "] {
        let prefix = text.strip_suffix("пишет ").unwrap();
        let first = format!("{prefix}пишу ");
        let second = format!("{prefix}пишут ");
        let (event, candidates, evaluations, admissions) =
            fixture(text, &[&first, &second], &[9.0, 2.0]);
        assert_eq!(
            select_with_readings(&event, &candidates, &evaluations, &admissions, lookup).selected,
            Some(0)
        );
    }
    let (event, candidates, evaluations, admissions) =
        fixture("они пишет ", &["я пишу ", "они пишут "], &[9.0, 2.0]);
    assert_eq!(
        select_with_readings(&event, &candidates, &evaluations, &admissions, lookup).reason,
        "agreement_not_current_word_replacement"
    );
    let text = "они вот вот вот вот вот вот вот вот пишет ";
    let prefix = text.strip_suffix("пишет ").unwrap();
    let first = format!("{prefix}пишу ");
    let second = format!("{prefix}пишут ");
    let (event, candidates, evaluations, admissions) =
        fixture(text, &[&first, &second], &[9.0, 2.0]);
    assert_eq!(
        select_with_readings(&event, &candidates, &evaluations, &admissions, lookup).selected,
        Some(0)
    );
}

#[test]
fn adjective_event_has_no_fabricated_future_noun() {
    let (event, candidates, evaluations, admissions) = fixture(
        "дом большой ",
        &["дом большую ", "дом большие "],
        &[9.0, 2.0],
    );
    let result = select_with_readings(&event, &candidates, &evaluations, &admissions, |surface| {
        let labels = match surface {
            "большой" => "adj:nom:sg:masc",
            "большую" => "adj:acc:sg:fem",
            "большие" => "adj:nom:pl",
            "дом" => "noun:nom:sg:masc",
            _ => return Some(Vec::new()),
        };
        Some(vec![identity(if surface == "дом" { 8 } else { 7 }, labels)])
    });
    assert_eq!(result.selected, Some(0));
    assert_eq!(result.reason, "agreement_no_observed_partner");
}

#[test]
fn retained_exact_and_closed_exact_absence_keep_original_precedence() {
    let (event, candidates, evaluations, admissions) =
        fixture("они пишет ", &["они пишу ", "они пишут "], &[9.0, 2.0]);
    let preferred =
        select_with_readings(&event, &candidates, &evaluations, &admissions, lookup).selected;
    assert_eq!(preferred, Some(1));
    assert_eq!(
        retained_exact_selection(preferred, RetainedExactDisposition::Valid(0)),
        Some(0)
    );
    assert_eq!(
        retained_exact_selection(preferred, RetainedExactDisposition::Invalid),
        None
    );
    let absent = TransitionDecisionCore::evaluate_candidates(
        &event,
        &candidates,
        TransitionDecisionPolicy::default(),
        DecisionEvidenceMode::ClosedExactAbsent,
    );
    assert!(absent.selected_index.is_none());
    assert!(absent.evaluations.is_empty());
    assert!(absent.selected_transition.is_none());
}
