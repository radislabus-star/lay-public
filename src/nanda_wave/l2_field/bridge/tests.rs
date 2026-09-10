use super::*;

#[test]
fn canonical_token_preserves_complete_physical_layout_surface() {
    assert_eq!(
        canonical_input_token("введи cj,frf "),
        Some(CanonicalInputToken {
            context_prefix: "введи ",
            surface: "cj,frf",
            kind: CanonicalInputTokenKind::PhysicalLayout,
        })
    );
    assert_eq!(
        canonical_input_token("ytn "),
        Some(CanonicalInputToken {
            context_prefix: "",
            surface: "ytn",
            kind: CanonicalInputTokenKind::PhysicalLayout,
        })
    );
}

#[test]
fn canonical_token_admits_lexically_supported_short_layout_surface() {
    assert_eq!(
        crate::dict::convert("yt", crate::dict::Direction::Us2Ru),
        "не"
    );
    assert_eq!(
        canonical_input_token("yt "),
        Some(CanonicalInputToken {
            context_prefix: "",
            surface: "yt",
            kind: CanonicalInputTokenKind::PhysicalLayout,
        })
    );
}

#[test]
fn canonical_token_preserves_cyrillic_context_identity() {
    assert_eq!(
        canonical_input_token("проверь врмея, "),
        Some(CanonicalInputToken {
            context_prefix: "проверь ",
            surface: "врмея",
            kind: CanonicalInputTokenKind::Cyrillic,
        })
    );
    assert_eq!(
        canonical_input_lexical_parts("проверь врмея, "),
        Some(("проверь ", "врмея"))
    );
}

#[test]
fn canonical_token_rejects_protected_ascii_classes() {
    for text in [
        "https://example.com ",
        "--help ",
        "release-build ",
        "PDF ",
        "Apple ",
    ] {
        assert_eq!(
            canonical_input_token(text),
            None,
            "protected ASCII token reached L1.1: {text:?}"
        );
    }
}

#[test]
fn canonical_token_rejects_known_english_plain_words() {
    assert!(crate::layout_autoswitch::is_known_english_layout_autoswitch_word("hello"));
    assert_eq!(canonical_input_token("hello "), None);
}

fn lexical_candidate(
    surface: &str,
    score: u32,
    l1_overlap: usize,
    l2_overlap: usize,
    motif_overlap: usize,
) -> crate::nanda_wave::l2::L2ImeWordCandidate {
    crate::nanda_wave::l2::L2ImeWordCandidate {
        surface: surface.to_string(),
        kind: crate::nanda_wave::l2::L2ImeWordCandidateKind::Replacement,
        source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
        score,
        l1_overlap,
        l2_overlap,
        motif_overlap,
        usage_prior: 0.0,
        context_prior: 0.0,
        accepted_count: 0,
        target_evidence: crate::nanda_wave::l2::L2ImeTargetEvidence::None,
        morphology_slots: Vec::new(),
    }
}

#[test]
fn canonical_readout_reserves_a_verified_two_content_boundary_candidate() {
    let readout = canonical_text_readout("Еленапросит ");
    let candidate = readout
        .candidates
        .iter()
        .find(|candidate| candidate.replacement == "Елена просит ")
        .expect("bounded boundary reserve");

    assert_eq!(candidate.origin, CandidateOrigin::Boundary);
    assert_eq!(candidate.source_id, "CanonicalL2FieldBoundary");
    assert_eq!(candidate.error_class, TypingErrorClass::GluedWords);
    assert!(candidate.has_l2_boundary_target_grounding());
}

#[test]
fn canonical_readout_reserves_a_strong_short_left_boundary_candidate() {
    let readout = canonical_text_readout("данорм ");
    let candidate = readout
        .candidates
        .iter()
        .find(|candidate| candidate.replacement == "да норм ")
        .expect("bounded short-left boundary reserve");

    assert_eq!(candidate.origin, CandidateOrigin::Boundary);
    assert_eq!(candidate.source_id, "CanonicalL2FieldBoundary");
    assert_eq!(candidate.error_class, TypingErrorClass::GluedWords);
    assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
    assert!(candidate.has_l2_boundary_target_grounding());
}

#[test]
fn canonical_readout_does_not_ground_unproven_fragment_splits() {
    for (original, unproven) in [
        ("относитться ", "относит ться "),
        ("Какие документаим ", "Какие документы им "),
    ] {
        let readout = canonical_text_readout(original);

        assert!(
            readout.candidates.iter().all(|candidate| {
                candidate.replacement != unproven || !candidate.has_l2_boundary_target_grounding()
            }),
            "an unproven or repaired split gained typed boundary grounding: {readout:#?}"
        );
    }
}

#[test]
fn canonical_readout_retains_grounded_split_despite_a_single_token_competitor() {
    let readout = canonical_text_readout("авторручка ");
    let candidate = readout
        .candidates
        .iter()
        .find(|candidate| candidate.replacement == "автор ручка ")
        .expect("target-grounded boundary candidate");

    assert_eq!(candidate.origin, CandidateOrigin::Boundary);
    assert_eq!(candidate.source_id, "CanonicalL2FieldBoundary");
    assert_eq!(candidate.error_class, TypingErrorClass::GluedWords);
    assert!(candidate.has_l2_boundary_target_grounding());
}

#[test]
fn sparse_omission_reserve_survives_below_the_general_frontier() {
    let surfaces = [
        "форма",
        "сигнал",
        "контур",
        "слово",
        "пакет",
        "дерево",
        "сцена",
        "волна",
        "центр",
        "подтвердить",
    ]
    .into_iter()
    .map(|surface| lexical_candidate(surface, 1_000, 1, 1, 1))
    .collect::<Vec<_>>();

    let bounded = bounded_surface_candidates_with_sparse_reserve("подврдить", &surfaces, 8, 2);

    assert_eq!(
        bounded
            .iter()
            .map(|candidate| candidate.surface.as_str())
            .collect::<Vec<_>>(),
        [
            "форма",
            "сигнал",
            "контур",
            "слово",
            "пакет",
            "дерево",
            "сцена",
            "волна",
            "подтвердить",
        ]
    );
    let mut unified =
        l2_surface_unified_candidates("на компанию Хунлу можем подврдить ", "подврдить", &bounded);
    apply_authority_to_candidate_lattice(&mut unified, &L2FieldAuthority::Abstain);
    let candidate = unified
        .iter()
        .find(|candidate| candidate.replacement.ends_with("подтвердить "))
        .expect("reserved sparse omission must reach the unified candidate lattice");
    assert_eq!(
        candidate.error_class,
        TypingErrorClass::SparseInternalMultiOmission
    );
    assert_eq!(candidate.gate.action, CandidateGateAction::SuggestOnly);
}

#[test]
fn composition_only_lattice_cannot_gain_authority() {
    assert_eq!(
        compositional_evidence_milli(Some(800), 750),
        600,
        "composition evidence must scale with the observed L1 peak"
    );
}

#[test]
fn non_l11_births_cannot_enter_the_l1_geometry_fallback() {
    let l11 = vec![lexical_candidate("форма", 1_000, 4, 0, 2)];
    let mut materialized = vec![
        lexical_candidate("форма", 1_000, 4, 0, 2),
        lexical_candidate("сигнал", 900, 5, 2, 3),
    ];

    assert_eq!(
        settle_unique_l1_geometry("сигна", &l11, &mut materialized, false),
        None
    );
}

#[test]
fn reference_backed_short_participle_ambiguity_blocks_false_singleton() {
    let mut surfaces = crate::ru_typo::fuzzy_known_word_candidates("подлючен")
        .into_iter()
        .filter(|surface| {
            crate::russian_lexicon::is_reference_backed_short_passive_participle(surface)
                && crate::text_metrics::damerau_levenshtein("подлючен", surface) == 1
        })
        .collect::<Vec<_>>();
    surfaces.sort();
    surfaces.dedup();
    assert!(surfaces.iter().any(|surface| surface == "подключен"));
    assert!(surfaces.iter().any(|surface| surface == "подлечен"));
    assert!(surfaces.len() >= 2);
    let candidates = surfaces
        .iter()
        .map(|surface| lexical_candidate(surface, 900, 6, 3, 3))
        .collect::<Vec<_>>();

    let readout = retain_reference_backed_geometry_ambiguity(
        "подлючен",
        CanonicalCohortReadout::Winner {
            winner_surface: "подключен".to_string(),
            cohort_surfaces: vec!["подключен".to_string()],
        },
        &candidates,
    );
    let CanonicalCohortReadout::Tied {
        mut cohort_surfaces,
    } = readout
    else {
        panic!("reference-backed one-edit ambiguity must not remain a singleton");
    };
    cohort_surfaces.sort();
    assert!(cohort_surfaces.iter().any(|surface| surface == "подключен"));
    assert!(cohort_surfaces.iter().any(|surface| surface == "подлечен"));
    assert!(cohort_surfaces.len() >= 2);

    let mut unified = l2_surface_unified_candidates("подлючен ", "подлючен", &candidates);
    demote_canonical_local_surface_cohort(
        &mut unified,
        &CanonicalCohortReadout::Tied { cohort_surfaces },
    );
    assert_eq!(unified.len(), candidates.len());
    assert!(unified
        .iter()
        .all(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly));
}

fn unified_candidate(
    replacement: &str,
    origin: CandidateOrigin,
    source_id: &str,
) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        source_id,
        TypingErrorClass::CompositeTypo,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "test",
        },
    )
}

fn morphology_evidence(
    lemma_id: u32,
    target_feature_mask: u32,
    generated: bool,
) -> MorphologySlotEvidence {
    MorphologySlotEvidence {
        lemma_id,
        source_feature_mask: 1,
        target_feature_mask,
        context_positive_support: 4,
        context_alternative_support: 1,
        context_posterior_milli: 800,
        slot_evidence_milli: 600,
        joint_evidence_milli: 900,
        generated,
    }
}

#[test]
fn typed_morphology_evidence_reaches_exact_and_generated_surfaces() {
    let mut candidates = vec![
        unified_candidate(
            "вы принуждаете ",
            CandidateOrigin::L2Surface,
            CANONICAL_L2_SURFACE_SOURCE_ID,
        ),
        unified_candidate(
            "вы принуждаетеся ",
            CandidateOrigin::L2Surface,
            CANONICAL_L2_SURFACE_SOURCE_ID,
        ),
    ];
    let productive_surfaces = ["принуждаетеся".to_string()].into_iter().collect();
    let evidence_by_surface = [
        (
            "принуждаете".to_string(),
            vec![morphology_evidence(17, 10, false)],
        ),
        (
            "принуждаетеся".to_string(),
            vec![morphology_evidence(17, 11, true)],
        ),
    ]
    .into_iter()
    .collect();

    mark_productive_surface_candidates(&mut candidates, &productive_surfaces, &evidence_by_surface);

    assert_eq!(candidates[0].source_id, CANONICAL_L2_SURFACE_SOURCE_ID);
    assert_eq!(candidates[0].morphology_slot_evidence.len(), 1);
    assert!(!candidates[0].morphology_slot_evidence[0].generated);
    assert_eq!(candidates[1].source_id, CANONICAL_L2_PRODUCTIVE_SOURCE_ID);
    assert_eq!(candidates[1].gate.action, CandidateGateAction::SuggestOnly);
    assert!(candidates[1].morphology_slot_evidence[0].generated);
}

#[test]
fn l1_geometry_settles_one_single_edit_peak() {
    let lexical = vec![
        lexical_candidate("время", 1889, 5, 0, 3),
        lexical_candidate("змея", 1782, 3, 0, 2),
    ];
    let mut materialized = lexical.clone();

    let readout = settle_unique_l1_geometry("врмея", &lexical, &mut materialized, true);

    assert_eq!(
        readout,
        Some(CanonicalCohortReadout::Winner {
            winner_surface: "время".to_string(),
            cohort_surfaces: vec!["время".to_string()],
        })
    );
    assert_eq!(materialized.len(), 1);
    assert_eq!(materialized[0].surface, "время");
}

#[test]
fn l1_geometry_keeps_multiple_single_edit_peaks_tied() {
    let lexical = vec![
        lexical_candidate("мзс", 1757, 2, 0, 3),
        lexical_candidate("мзд", 1743, 2, 0, 3),
    ];
    let mut materialized = lexical.clone();

    let readout = settle_unique_l1_geometry("мзт", &lexical, &mut materialized, true);

    assert_eq!(
        readout,
        Some(CanonicalCohortReadout::Tied {
            cohort_surfaces: vec!["мзд".to_string(), "мзс".to_string()],
        })
    );
    assert_eq!(materialized.len(), 2);
}

#[test]
fn abstain_demotes_all_owned_surface_candidates_not_only_reported_cohort() {
    let mut candidates = vec![
        unified_candidate(
            "проверка ",
            CandidateOrigin::L2Surface,
            CANONICAL_L2_SURFACE_SOURCE_ID,
        ),
        unified_candidate(
            "проварка ",
            CandidateOrigin::L2Surface,
            CANONICAL_L2_SURFACE_SOURCE_ID,
        ),
    ];
    let readout = CanonicalCohortReadout::Abstain {
        cohort_surfaces: vec!["проверка".to_string()],
    };

    demote_canonical_local_surface_cohort(&mut candidates, &readout);

    assert!(candidates
        .iter()
        .all(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly));
}

#[test]
fn abstain_preserves_independent_deterministic_typo_and_layout_authority() {
    let mut candidates = vec![
        unified_candidate(
            "понимаешь ",
            CandidateOrigin::DeterministicTypo,
            "composite_ru_typo",
        ),
        unified_candidate("vpn ", CandidateOrigin::Layout, "layout_ru_to_en"),
    ];

    apply_authority_to_candidate_lattice(&mut candidates, &L2FieldAuthority::Abstain);

    assert_eq!(candidates[0].gate.action, CandidateGateAction::Eligible);
    assert_eq!(candidates[1].gate.action, CandidateGateAction::Eligible);
}

#[test]
fn unavailable_preserves_independent_deterministic_typo_and_layout_authority() {
    let mut candidates = vec![
        unified_candidate(
            "Приши ",
            CandidateOrigin::DeterministicTypo,
            "missing_letter",
        ),
        unified_candidate("pdf ", CandidateOrigin::Layout, "layout_ru_to_en"),
    ];

    apply_authority_to_candidate_lattice(&mut candidates, &L2FieldAuthority::Unavailable);

    assert_eq!(candidates[0].gate.action, CandidateGateAction::Eligible);
    assert_eq!(candidates[1].gate.action, CandidateGateAction::Eligible);
}

#[test]
fn noncontradictory_field_absence_still_demotes_l2_only_evidence() {
    for authority in [L2FieldAuthority::Abstain, L2FieldAuthority::Unavailable] {
        let mut candidates = vec![UnifiedCorrectionCandidate::new(
            "проверка ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            CANONICAL_L2_SURFACE_SOURCE_ID,
            TypingErrorClass::MissingLetter,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "test",
            },
        )];

        apply_authority_to_candidate_lattice(&mut candidates, &authority);

        assert_eq!(candidates[0].gate.action, CandidateGateAction::SuggestOnly);
    }
}

#[test]
fn noncontradictory_field_absence_never_promotes_a_deterministic_suggestion() {
    let mut candidate = unified_candidate(
        "проверка ",
        CandidateOrigin::DeterministicTypo,
        "composite_ru_typo",
    );
    candidate.gate = CandidateGateDecision {
        action: CandidateGateAction::SuggestOnly,
        reason: "test_requires_more_evidence",
    };
    candidate.evidence[0].gate = candidate.gate.clone();

    apply_authority_to_candidate_lattice(
        std::slice::from_mut(&mut candidate),
        &L2FieldAuthority::Abstain,
    );

    assert_eq!(candidate.gate.action, CandidateGateAction::SuggestOnly);
    assert_eq!(candidate.gate.reason, "test_requires_more_evidence");
}

#[test]
fn abstain_preserves_a_merged_surface_with_eligible_deterministic_evidence() {
    let mut candidate = UnifiedCorrectionCandidate::new(
        "проверка ",
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        CANONICAL_L2_SURFACE_SOURCE_ID,
        TypingErrorClass::MissingLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "test_l2",
        },
    );
    candidate.merge_evidence(unified_candidate(
        "проверка ",
        CandidateOrigin::DeterministicTypo,
        "missing_letter",
    ));

    apply_authority_to_candidate_lattice(
        std::slice::from_mut(&mut candidate),
        &L2FieldAuthority::Abstain,
    );

    assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
    assert!(candidate.has_eligible_origin(CandidateOrigin::DeterministicTypo));
    assert!(!candidate.has_eligible_origin(CandidateOrigin::L2Surface));
}

#[test]
fn winner_owns_lexical_authority_case_insensitively() {
    let mut candidates = vec![
        unified_candidate(
            "Посмотри ",
            CandidateOrigin::DeterministicTypo,
            "missing_letter",
        ),
        unified_candidate(
            "Посмотреть ",
            CandidateOrigin::DeterministicTypo,
            "missing_letter",
        ),
    ];

    apply_authority_to_candidate_lattice(
        &mut candidates,
        &L2FieldAuthority::Winner {
            surface: "посмотри".to_string(),
        },
    );

    assert_eq!(candidates[0].gate.action, CandidateGateAction::Eligible);
    assert_eq!(candidates[1].gate.action, CandidateGateAction::SuggestOnly);
}

#[test]
fn tie_demotes_ambiguous_length_change_but_preserves_other_verified_member() {
    let mut candidates = vec![
        unified_candidate(
            "перехвачу ",
            CandidateOrigin::DeterministicTypo,
            "missing_letter",
        ),
        unified_candidate(
            "передачу ",
            CandidateOrigin::DeterministicTypo,
            "composite_ru_typo",
        ),
    ];
    candidates[0].error_class = TypingErrorClass::MissingLetter;
    candidates[1].error_class = TypingErrorClass::LetterSubstitution;
    candidates.push(unified_candidate(
        "перехвачу ",
        CandidateOrigin::DeterministicTypo,
        "vowel_confusion",
    ));
    candidates[2].error_class = TypingErrorClass::LetterSubstitution;
    candidates.push(unified_candidate(
        "перехват ",
        CandidateOrigin::DeterministicTypo,
        "missing_letter",
    ));

    apply_authority_to_candidate_lattice(
        &mut candidates,
        &L2FieldAuthority::Tied {
            surfaces: vec!["первачу".to_string(), "перехвачу".to_string()],
        },
    );

    assert_eq!(candidates[0].gate.action, CandidateGateAction::SuggestOnly);
    assert_eq!(candidates[1].gate.action, CandidateGateAction::SuggestOnly);
    assert_eq!(candidates[2].gate.action, CandidateGateAction::Eligible);
    assert_eq!(candidates[3].gate.action, CandidateGateAction::SuggestOnly);
}
