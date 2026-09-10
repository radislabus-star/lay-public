use super::*;

#[test]
fn punctuation_does_not_hide_word_form_authority() {
    let candidate = WordCandidate {
        text: " отстранилась!".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: LEXICAL_ATTRACTOR_CELL,
        energy: 0.95,
        risk: 0.10,
        support: vec![],
    };

    assert!(!word_form_candidate_lacks_surface_support(
        " отсранилась! ",
        &candidate,
        TypingErrorClass::MissingLetter,
    ));
}

#[test]
fn l3_context_support_can_select_sparse_internal_omission_center() {
    let original = "ты записал нашу новую концепцию интелека ";
    let candidates = [
        WordCandidate {
            text: "ты записал нашу новую концепцию интелект".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.106,
            support: vec![],
        },
        WordCandidate {
            text: "ты записал нашу новую концепцию интеллекта".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.912,
            risk: 0.172,
            support: vec!["l2-operator:sparse-internal-multi-omission".to_string()],
        },
    ];
    let reports = [
        None,
        Some(l3_phrase_gate::L3PhraseGateReport {
            decision: l3_phrase_gate::L3PhraseGateDecision::Support,
            source: "learned_context_phase",
            score: 0.179,
            rank_energy: 0.028,
            support: 2,
            width: 5,
            sequential_score: 0.179,
            scene_score: 2.0,
            competition_margin: 0.211,
            positive_micro: 179_000,
            anti_micro: 0,
            threshold_micro: 700_000,
            relation_class: 1,
            pairwise_certified: false,
            reason: "l3_context_phase_support",
        }),
    ];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn same_script_l2_repair_beats_cross_script_layout_projection() {
    let original = "ландо ";
    let candidates = [
        WordCandidate {
            text: "ладно".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.104,
            support: vec!["l2-operator:adjacent-transposition".to_string()],
        },
        WordCandidate {
            text: "kayla".to_string(),
            origin: CandidateOrigin::LayoutThenTypo,
            source: "layout_then_l2_word_center",
            energy: 0.864,
            risk: 0.160,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn internal_char_confusion_is_typed_l2_damage_not_word_drift() {
    let original = "абоенет ";
    let candidates = [
        WordCandidate {
            text: "абонент".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.226,
            support: vec![],
        },
        WordCandidate {
            text: "кабинет".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.264,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert!(!word_form_candidate_lacks_surface_support(
        original,
        &candidates[0],
        TypingErrorClass::CompositeTypo,
    ));
    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn bounded_surface_frame_repair_gets_word_form_authority() {
    let original = "приимущестов ";
    let candidate = WordCandidate {
        text: "преимущество".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.95,
        risk: 0.170,
        support: vec![],
    };

    assert!(!word_form_candidate_lacks_surface_support(
        original,
        &candidate,
        TypingErrorClass::CompositeTypo,
    ));
}

#[test]
fn weak_surface_frame_repair_stays_blocked() {
    let candidate = WordCandidate {
        text: "абразия".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.781,
        risk: 0.260,
        support: vec![],
    };

    assert!(!bounded_l2_surface_frame_repair_has_authority(
        "абареия",
        "абразия",
        &candidate,
        2,
        2,
        false,
    ));
}

#[test]
fn known_surface_still_allows_verified_internal_missing_letter_repair() {
    let original = "вобще ";
    let candidate = WordCandidate {
        text: "вообще".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.95,
        risk: 0.04,
        support: vec!["l2-operator:single-internal-missing-letter".to_string()],
    };

    assert!(!word_form_candidate_lacks_surface_support(
        original,
        &candidate,
        TypingErrorClass::MissingLetter,
    ));
}

#[test]
fn dictionary_word_does_not_become_missing_letter_repair_without_context_proof() {
    let original = "вышли ";
    let candidate = WordCandidate {
        text: "вышили".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.95,
        risk: 0.107,
        support: vec!["l2-operator:single-internal-missing-letter".to_string()],
    };

    assert!(word_form_candidate_lacks_surface_support(
        original,
        &candidate,
        TypingErrorClass::MissingLetter,
    ));
}

#[test]
fn clipped_surface_allows_single_missing_letter_repair_to_stable_center() {
    let original = "можн ";
    let candidate = WordCandidate {
        text: "можно".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.95,
        risk: 0.04,
        support: vec!["l2-operator:single-missing-letter".to_string()],
    };

    assert!(!word_form_candidate_lacks_surface_support(
        original,
        &candidate,
        TypingErrorClass::MissingLetter,
    ));
}

#[test]
fn prefix_completion_center_beats_destructive_short_typo_competitor() {
    let original = "абсу ";
    let candidates = [
        WordCandidate {
            text: "абсурд".to_string(),
            origin: CandidateOrigin::Completion,
            source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
            energy: 0.95,
            risk: 0.06,
            support: vec![],
        },
        WordCandidate {
            text: "басу".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.854,
            risk: 0.10,
            support: vec!["l2-operator:adjacent-transposition".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn known_dictionary_surface_blocks_unframed_repeated_collapse() {
    let original = "исправленно ";
    let candidates = [
        WordCandidate {
            text: "исправлено".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.125,
            support: vec!["l2-operator:repeated-letter-collapse".to_string()],
        },
        WordCandidate {
            text: "исправленном".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.117,
            support: vec!["l2-operator:single-missing-letter".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        context_candidate_pre_phrase_blocker(original, &candidates[0]),
        Some("word_form_authority")
    );
    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn orthographic_sign_repair_is_typed_damage_not_word_drift() {
    let original = "Обьясни ";
    let candidate = WordCandidate {
        text: "Объясни".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.733,
        risk: 0.147,
        support: vec!["l2-operator:orthographic-sign-repair".to_string()],
    };

    assert!(!word_form_candidate_lacks_surface_support(
        original,
        &candidate,
        TypingErrorClass::CompositeTypo,
    ));
}

#[test]
fn boundary_split_with_weak_tail_yields_to_current_token_repair() {
    let original = "кторое ";
    let candidates = [
        WordCandidate {
            text: "к торое".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["hidden-short-function-boundary".to_string()],
        },
        WordCandidate {
            text: "которое".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.115,
            support: vec!["l2-operator:single-internal-missing-letter".to_string()],
        },
    ];
    let reports = [None, None];

    assert!(crate::text_metrics::current_token_boundary_split(
        original,
        &candidates[0].text,
    ));
    assert_eq!(
        current_token_boundary_split_parts(original, &candidates[0].text),
        Some(("к".to_string(), "торое".to_string()))
    );
    assert!(boundary_split_tail_is_weak("торое"));
    assert!(boundary_split_should_yield_to_current_token_repair(
        original,
        &candidates[0],
        &candidates,
    ));
    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn boundary_split_yields_to_repeated_letter_repair() {
    let original = "аабсент ";
    let candidates = [
        WordCandidate {
            text: "а абсент".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["hidden-short-function-boundary".to_string()],
        },
        WordCandidate {
            text: "абсент".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.101,
            support: vec!["l2-operator:repeated-letter-collapse".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn boundary_split_still_beats_word_drift_with_known_tail() {
    let original = "влогах ";
    let candidates = [
        WordCandidate {
            text: "в логах".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["hidden-short-function-boundary".to_string()],
        },
        WordCandidate {
            text: "волгах".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.921,
            risk: 0.121,
            support: vec!["l2-operator:adjacent-transposition".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn repaired_boundary_split_beats_whole_word_lexical_drift() {
    let original = "прблематут ";
    let candidates = [
        WordCandidate {
            text: "проблема тут".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["light-boundary-split".to_string()],
        },
        WordCandidate {
            text: "проблематик".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.11,
            support: vec!["l2-operator:internal-extra-fragment".to_string()],
        },
    ];
    let reports = [None, None];

    assert!(crate::text_metrics::current_token_boundary_split_or_repair(
        original,
        &candidates[0].text,
    ));
    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn boundary_split_pressure_beats_destructive_whole_word_shortening() {
    let original = "вотидело ";
    let candidates = [
        WordCandidate {
            text: "видео".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 1.0,
            risk: 0.099,
            support: vec!["l2-operator:internal-extra-fragment".to_string()],
        },
        WordCandidate {
            text: "вот и дело".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["light-boundary-split".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn repaired_boundary_with_weak_tail_yields_to_typed_word_repair() {
    let original = "рабоатет ";
    let candidates = [
        WordCandidate {
            text: "работа тет".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["light-boundary-split".to_string()],
        },
        WordCandidate {
            text: "работает".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.04,
            support: vec!["l2-operator:adjacent-transposition".to_string()],
        },
    ];
    let reports = [None, None];

    assert!(boundary_split_should_yield_to_current_token_repair(
        original,
        &candidates[0],
        &candidates,
    ));
    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn exact_two_content_center_split_gets_boundary_authority() {
    let original = "самоетоже ";
    let candidates = [
        WordCandidate {
            text: "самое тоже".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["light-boundary-split".to_string()],
        },
        WordCandidate {
            text: "смоете".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.12,
            support: vec!["l2-operator:internal-extra-fragment".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn single_missing_repair_beats_sparse_expansion_competitor() {
    let original = "прорватся ";
    let candidates = [
        WordCandidate {
            text: "прорваться".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.081,
            support: vec!["l2-operator:single-internal-missing-letter".to_string()],
        },
        WordCandidate {
            text: "прорываться".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.111,
            support: vec!["l2-operator:sparse-internal-multi-omission".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn adjacent_transposition_beats_missing_letter_expansion() {
    let original = "абиджна ";
    let candidates = [
        WordCandidate {
            text: "абиджан".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.090,
            support: vec!["l2-operator:adjacent-transposition".to_string()],
        },
        WordCandidate {
            text: "абиджана".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.115,
            support: vec!["l2-operator:single-internal-missing-letter".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn single_letter_substitution_beats_missing_letter_expansion() {
    let original = "видешь ";
    let candidates = [
        WordCandidate {
            text: "видишь".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.065,
            support: vec!["l2-operator:single-letter-substitution".to_string()],
        },
        WordCandidate {
            text: "видаешь".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.107,
            support: vec!["l2-operator:single-internal-missing-letter".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn inferred_sparse_repair_beats_destructive_boundary_split() {
    let original = "высокопными ";
    let candidates = [
        WordCandidate {
            text: "высоко паными".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["light-boundary-split".to_string()],
        },
        WordCandidate {
            text: "высокопарными".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.926,
            risk: 0.252,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn boundary_split_stays_when_no_typed_current_token_repair_exists() {
    let original = "онаубыточная ";
    let candidates = [
        WordCandidate {
            text: "она убыточная".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.99,
            risk: 0.04,
            support: vec!["hidden-short-function-boundary".to_string()],
        },
        WordCandidate {
            text: "безубыточная".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.892,
            risk: 0.312,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn blocked_same_script_shadow_cannot_hide_direct_layout_projection() {
    let original = "сркщьу ";
    let candidates = [
        WordCandidate {
            text: "сразу".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 1.0,
            risk: 0.14,
            support: vec![],
        },
        WordCandidate {
            text: "chrome".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.906,
            risk: 0.030,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn direct_layout_projection_beats_weak_same_script_shadow() {
    let original = "ашду ";
    let candidates = [
        WordCandidate {
            text: "аиду".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.793,
            risk: 0.210,
            support: vec![],
        },
        WordCandidate {
            text: "file".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.929,
            risk: 0.030,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn technical_direct_layout_projection_beats_same_script_shadow() {
    let original = "реьд ";
    let candidates = [
        WordCandidate {
            text: "рейд".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.935,
            risk: 0.166,
            support: vec![],
        },
        WordCandidate {
            text: "html".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.872,
            risk: 0.030,
            support: vec![],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn typed_damage_support_can_repair_stable_surface_artifact() {
    let candidate = WordCandidate {
        text: "найди".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: super::super::l2::L2_SURFACE_MOTIF_CELL,
        energy: 0.95,
        risk: 0.07,
        support: vec!["l2-operator:adjacent-transposition".to_string()],
    };

    assert!(!word_form_candidate_lacks_surface_support(
        "надйи ",
        &candidate,
        TypingErrorClass::AdjacentTransposition,
    ));
}

#[test]
fn known_dictionary_surface_withholds_unframed_morphology_and_typed_drift() {
    let original = "исправленно ";
    let candidates = [
        WordCandidate {
            text: "исправление".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 1.0,
            risk: 0.136,
            support: vec![],
        },
        WordCandidate {
            text: "исправлено".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.107,
            support: vec!["l2-operator:repeated-letter-collapse".to_string()],
        },
    ];
    let reports = [None, None];

    assert!(candidates.iter().all(|candidate| {
        context_candidate_pre_phrase_blocker(original, candidate) == Some("word_form_authority")
    }));
    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        None
    );
}

#[test]
fn sparse_omission_lattice_prefers_preserved_typed_prefix() {
    let original = "испрть ";
    let candidates = [
        WordCandidate {
            text: "испарить".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.137,
            support: vec!["l2-operator:sparse-internal-multi-omission".to_string()],
        },
        WordCandidate {
            text: "исправить".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.264,
            support: vec!["l2-operator:sparse-internal-multi-omission".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn sparse_omission_lattice_handles_real_noisy_competitor_pack() {
    let original = "испрть ";
    let words = [
        ("купить", 1.000, 0.158, vec![]),
        ("испр", 1.000, 0.170, vec![]),
        (
            "испарить",
            0.950,
            0.137,
            vec!["l2-operator:sparse-internal-multi-omission"],
        ),
        (
            "испортить",
            0.950,
            0.149,
            vec!["l2-operator:sparse-internal-multi-omission"],
        ),
        ("исправить", 0.950, 0.264, vec![]),
        ("испытать", 0.950, 0.264, vec![]),
        ("исправь", 0.939, 0.180, vec![]),
    ];
    let candidates = words
        .into_iter()
        .map(|(text, energy, risk, support)| WordCandidate {
            text: text.to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy,
            risk,
            support: support.into_iter().map(str::to_string).collect(),
        })
        .collect::<Vec<_>>();
    let reports = vec![None; candidates.len()];

    let index = best_context_candidate(original, &candidates, &reports)
        .expect("real sparse-omission pack should have a candidate");
    assert_eq!(candidates[index].text, "исправить");
}

#[test]
fn broad_attractor_suffix_drift_needs_phase_or_typed_operator() {
    let candidate = WordCandidate {
        text: "кодированием".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: LEXICAL_ATTRACTOR_CELL,
        energy: 0.95,
        risk: 0.252,
        support: vec![],
    };

    assert!(word_form_candidate_lacks_surface_support(
        "кодировании ",
        &candidate,
        TypingErrorClass::Unknown,
    ));
}

#[test]
fn context_support_cannot_jump_past_the_local_transition_field() {
    let original = "на улице снова начался дожь ";
    let local = WordCandidate {
        text: "на улице снова начался дождь".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: LEXICAL_ATTRACTOR_CELL,
        energy: 0.95,
        risk: 0.06,
        support: vec![],
    };
    let distant = WordCandidate {
        text: "на улице снова начался до".to_string(),
        origin: CandidateOrigin::L2Surface,
        source: LEXICAL_ATTRACTOR_CELL,
        energy: 0.95,
        risk: 0.10,
        support: vec![],
    };
    let report = l3_phrase_gate::L3PhraseGateReport {
        decision: l3_phrase_gate::L3PhraseGateDecision::Support,
        source: "learned_context_phase",
        score: 0.50,
        rank_energy: 0.08,
        support: 8,
        width: 4,
        sequential_score: 0.50,
        scene_score: 8.0,
        competition_margin: 0.10,
        positive_micro: 500_000,
        anti_micro: 0,
        threshold_micro: 300_000,
        relation_class: 1,
        pairwise_certified: false,
        reason: "l3_context_phase_support",
    };

    assert_eq!(context_transition_distance(original, &local.text), Some(1));
    assert!(
        context_transition_distance(original, &distant.text).is_some_and(|distance| distance > 1)
    );
    assert!(!context_support_is_transition_local(
        original,
        &distant,
        Some(&report),
        Some(1),
    ));
}

#[test]
fn tracked_l3_context_scores_the_full_real_l2_lattice() {
    let original = "ты записал нашу новую концепцию интелека ";
    let l1 = super::super::l1::run_l1(original);
    let candidates = super::super::l2::run_l2(original, &l1);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package =
        super::super::context_phase::read_package(&path).expect("tracked L3 context phase package");
    let replacements = candidates
        .iter()
        .map(|candidate| candidate.text.as_str())
        .collect::<Vec<_>>();
    let context = llmwave::tokenize(original).len().saturating_sub(1);
    let reports = l3_phrase_gate::reports_from_phase_readouts(
        context,
        super::super::context_phase::readout_candidates_with_package(
            &package,
            original,
            &replacements,
        ),
    );

    assert_eq!(reports.len(), candidates.len());
    let target = candidates
        .iter()
        .position(|candidate| candidate.text == "ты записал нашу новую концепцию интеллекта")
        .expect("L2 must expose the sparse-omission target");
    assert!(
        reports[target].is_some(),
        "the context field must observe a context-preserving L2 candidate"
    );
    assert!(
        context_candidate_blocker(original, &candidates[target], reports[target].as_ref())
            .is_none(),
        "sparse internal omission target should pass the typed transition verifier"
    );
}

#[test]
fn applies_confident_layout_candidate() {
    let candidate = WordCandidate {
        text: "html вот".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.8,
        risk: 0.1,
        support: vec![],
    };
    let (_trace, decision) = run_l3("html djn ", &[candidate]);
    assert_eq!(decision.output(), Some("html вот "));
}

#[test]
fn applies_verified_single_word_layout_candidate() {
    let candidate = WordCandidate {
        text: "проверь".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 1.0,
        risk: 0.05,
        support: vec![],
    };

    let (_trace, decision) = run_l3("ghjdthm", &[candidate]);

    assert_eq!(decision.output(), Some("проверь"));
}

#[test]
fn l3_ranks_short_prefix_completion_without_granting_apply_authority() {
    let candidates = [
        WordCandidate {
            text: "Ну давай".to_string(),
            origin: CandidateOrigin::Completion,
            source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
            energy: 0.856,
            risk: 0.060,
            support: vec![],
        },
        WordCandidate {
            text: "Ну даша".to_string(),
            origin: CandidateOrigin::Completion,
            source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
            energy: 0.856,
            risk: 0.060,
            support: vec![],
        },
    ];

    let (_trace, decision) = run_l3("Ну да ", &candidates);

    assert_eq!(decision.output(), Some("Ну давай "));
}

#[test]
fn l3_keeps_first_equal_candidate_instead_of_last_equal_candidate() {
    let candidates = [
        WordCandidate {
            text: "попаданий".to_string(),
            origin: CandidateOrigin::L3Context,
            source: SEMANTIC_WORD_SOURCE,
            energy: 0.80,
            risk: 0.10,
            support: vec![],
        },
        WordCandidate {
            text: "попадали".to_string(),
            origin: CandidateOrigin::L3Context,
            source: SEMANTIC_WORD_SOURCE,
            energy: 0.80,
            risk: 0.10,
            support: vec![],
        },
    ];

    let (_trace, decision) = run_l3("попадани ", &candidates);

    assert_eq!(decision.output(), Some("попаданий "));
}

#[test]
fn l3_exposes_surface_completion_as_non_mutating_readout() {
    let candidate = WordCandidate {
        text: "попаданий".to_string(),
        origin: CandidateOrigin::Completion,
        source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
        energy: 0.856,
        risk: 0.060,
        support: vec![],
    };

    let (_trace, decision) = run_l3("попадани ", &[candidate]);

    assert_eq!(decision.output(), Some("попаданий "));
}

#[test]
fn keeps_short_layout_candidate_without_russian_phrase_context() {
    let candidate = WordCandidate {
        text: "wave и".to_string(),
        origin: CandidateOrigin::Layout,
        source: "ShortTokenCell32",
        energy: 0.9,
        risk: 0.46,
        support: vec![],
    };
    let (_trace, decision) = run_l3("wave b ", &[candidate]);

    assert_eq!(decision.output(), None);
    assert_eq!(
        decision,
        WaveDecision::Keep {
            reason: "short_layout_without_phrase_context"
        }
    );
}

#[test]
fn l3_weight_scales_structural_boosts() {
    let candidate = WordCandidate {
        text: "html вот".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.32,
        risk: 0.10,
        support: vec![],
    };
    let (_trace, muted) = run_l3_with_options(
        "html djn ",
        std::slice::from_ref(&candidate),
        &WaveOptions::default().with_layer_weights(1.0, 0.0),
    );
    let (_trace, normal) = run_l3_with_options("html djn ", &[candidate], &WaveOptions::default());

    assert_ne!(muted, normal);
}

#[test]
fn applies_boundary_candidate() {
    let candidate = WordCandidate {
        text: "у нас есть".to_string(),
        origin: CandidateOrigin::Boundary,
        source: "BoundaryCell32",
        energy: 0.8,
        risk: 0.1,
        support: vec![],
    };
    let (_trace, decision) = run_l3("у насесть ", &[candidate]);
    assert_eq!(decision.output(), Some("у нас есть "));
}

#[test]
fn technical_candidate_vetoes_layout_candidate() {
    let technical = WordCandidate {
        text: "git checkout -b new".to_string(),
        origin: CandidateOrigin::Technical,
        source: "TechTokenCell32",
        energy: 0.95,
        risk: 0.02,
        support: vec![],
    };
    let layout = WordCandidate {
        text: "git checkout -b туц".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.8,
        risk: 0.1,
        support: vec![],
    };
    let (_trace, decision) = run_l3("git checkout -b new ", &[technical, layout]);
    assert_eq!(decision.output(), None);
}

#[test]
fn phrase_layout_candidate_cannot_rewrite_middle_token() {
    let technical = WordCandidate {
        text: "api".to_string(),
        origin: CandidateOrigin::Technical,
        source: "TechTokenCell32",
        energy: 0.95,
        risk: 0.02,
        support: vec![],
    };
    let layout = WordCandidate {
        text: "html вот api".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.8,
        risk: 0.1,
        support: vec![],
    };
    let (_trace, decision) = run_l3("html djn api ", &[technical, layout]);
    assert_eq!(decision.output(), None);
}

#[test]
fn applies_split_memory_candidate() {
    let candidate = WordCandidate {
        text: "она есть".to_string(),
        origin: CandidateOrigin::Boundary,
        source: "PhraseMemoryCell32",
        energy: 0.82,
        risk: 0.11,
        support: vec![],
    };
    let (_trace, decision) = run_l3("онаесть ", &[candidate]);
    assert_eq!(decision.output(), Some("она есть "));
}

#[test]
fn phrase_forecast_boosts_semantic_candidate() {
    let candidate = WordCandidate {
        text: "На улице опять идёт дождь".to_string(),
        origin: CandidateOrigin::Completion,
        source: PHRASE_FORECAST_CELL,
        energy: 0.30,
        risk: 0.10,
        support: vec![],
    };
    let (_trace, decision) = run_l3("На улице опять идёт д ", &[candidate]);
    assert_eq!(decision.output(), Some("На улице опять идёт дождь "));
}

#[test]
fn semantic_word_candidate_needs_surface_authority() {
    let candidate = WordCandidate {
        text: "она спрашивая".to_string(),
        origin: CandidateOrigin::L3Context,
        source: SEMANTIC_WORD_SOURCE,
        energy: 0.90,
        risk: 0.10,
        support: vec![],
    };
    let (_trace, decision) = run_l3("она спраивтя ", &[candidate]);

    assert_eq!(decision.output(), None);
}

#[test]
fn semantic_word_completion_keeps_l3_authority() {
    let candidate = WordCandidate {
        text: "на улице опять идёт дождь".to_string(),
        origin: CandidateOrigin::L3Context,
        source: SEMANTIC_WORD_SOURCE,
        energy: 0.50,
        risk: 0.10,
        support: vec![],
    };
    let (_trace, decision) = run_l3("на улице опять идёт д ", &[candidate]);

    assert_eq!(decision.output(), Some("на улице опять идёт дождь "));
}

#[test]
fn l3_phrase_memory_reranks_competing_l2_candidates() {
    let memory = llmwave::LlmWaveMemory::from_text(
        "на улице опять идёт дождь\nсегодня на улице опять идёт дождь\nвечером на улице опять идёт дождь\nзавтра на улице опять идёт дождь",
    );
    let candidates = vec![
        WordCandidate {
            text: "на улице опять идёт дом".to_string(),
            origin: CandidateOrigin::L3Context,
            source: SEMANTIC_WORD_SOURCE,
            energy: 0.86,
            risk: 0.06,
            support: vec![],
        },
        WordCandidate {
            text: "на улице опять идёт дождь".to_string(),
            origin: CandidateOrigin::L3Context,
            source: SEMANTIC_WORD_SOURCE,
            energy: 0.42,
            risk: 0.08,
            support: vec![],
        },
    ];
    let (trace, decision) = run_l3_inner(
        "на улице опять идёт д ",
        &candidates,
        &WaveOptions::default(),
        Some(&memory),
    );

    assert_eq!(decision.output(), Some("на улице опять идёт дождь "));
    assert!(trace
        .iter()
        .any(|item| item.summary.contains("l3_phrase=l3_context_field_support")));

    let without_context = WaveOptions::with_disabled(&[L3_CONTEXT_FIELD_CELL.to_string()]);
    let (_trace, decision) = run_l3_inner(
        "на улице опять идёт д ",
        &candidates,
        &without_context,
        Some(&memory),
    );
    assert_eq!(decision.output(), Some("на улице опять идёт дом "));

    let zero_weight = WaveOptions::default().with_layer_weights(1.0, 0.0);
    let (_trace, decision) = run_l3_inner(
        "на улице опять идёт д ",
        &candidates,
        &zero_weight,
        Some(&memory),
    );
    assert_eq!(decision.output(), Some("на улице опять идёт дом "));
}

#[test]
fn pattern_wave_is_visible_in_l3_trace() {
    let candidate = WordCandidate {
        text: "html вот".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.55,
        risk: 0.25,
        support: vec![],
    };
    let (trace, decision) = run_l3("html djn ", &[candidate]);

    assert!(trace
        .iter()
        .any(|item| item.name == super::super::pattern_wave::PATTERN_WAVE_CELL));
    assert_eq!(decision.output(), Some("html вот "));
}

#[test]
fn pattern_wave_can_veto_layout_candidate_in_technical_shape() {
    let candidate = WordCandidate {
        text: "git вот".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.95,
        risk: 0.05,
        support: vec![],
    };
    let (_trace, decision) = run_l3("git djn ", &[candidate]);

    assert_eq!(decision.output(), None);
}

#[test]
fn structural_relation_is_visible_in_l3_trace() {
    let candidate = WordCandidate {
        text: "пишу вот".to_string(),
        origin: CandidateOrigin::Layout,
        source: "LayoutWordCell32",
        energy: 0.50,
        risk: 0.27,
        support: vec![],
    };
    let options = WaveOptions::with_disabled(&[L3_CONTEXT_FIELD_CELL.to_string()]);
    let (trace, decision) = run_l3_with_options("пишу djn ", &[candidate], &options);

    assert!(
        trace
            .iter()
            .any(|item| item.name == super::super::structural_relation::STRUCTURAL_RELATION_CELL),
        "unexpected L3 trace: {trace:#?}"
    );
    assert_eq!(decision.output(), Some("пишу вот "));
}

#[test]
fn reference_prior_breaks_missing_vs_repeated_tie() {
    let original = "аажур ";
    let candidates = [
        WordCandidate {
            text: "абажур".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.101,
            support: vec!["l2-operator:single-internal-missing-letter".to_string()],
        },
        WordCandidate {
            text: "ажур".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.101,
            support: vec!["l2-operator:repeated-letter-collapse".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(0)
    );
}

#[test]
fn reference_prior_marks_unframed_suffix_substitution_as_weaker() {
    let original = "дальг ";
    let candidates = [
        WordCandidate {
            text: "далью".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.147,
            support: vec!["l2-operator:single-letter-substitution".to_string()],
        },
        WordCandidate {
            text: "дальше".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.903,
            risk: 0.261,
            support: vec![],
        },
    ];

    assert!(lexical_reference_prior("дальше") > lexical_reference_prior("далью"));
    assert!(unframed_substitution_competition_pressure(original, &candidates[0], true) < 0.0);
}

#[test]
fn reference_prior_can_beat_inflected_transposition_competitor() {
    let original = "абдомеен ";
    let candidates = [
        WordCandidate {
            text: "абдомене".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.090,
            support: vec!["l2-operator:adjacent-transposition".to_string()],
        },
        WordCandidate {
            text: "абдомен".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.115,
            support: vec!["l2-operator:repeated-letter-collapse".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

#[test]
fn missing_letter_repair_beats_unframed_substitution() {
    let original = "другие перемнные ";
    let candidates = [
        WordCandidate {
            text: "другие переэнные".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.085,
            support: vec!["l2-operator:single-letter-substitution".to_string()],
        },
        WordCandidate {
            text: "другие переменные".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: super::super::l2::L2_SURFACE_MOTIF_CELL,
            energy: 0.95,
            risk: 0.040,
            support: vec!["l2-operator:single-internal-missing-letter".to_string()],
        },
    ];
    let reports = [None, None];

    assert_eq!(
        best_context_candidate(original, &candidates, &reports),
        Some(1)
    );
}

trait DecisionOutput {
    fn output(&self) -> Option<&str>;
}

impl DecisionOutput for WaveDecision {
    fn output(&self) -> Option<&str> {
        match self {
            WaveDecision::Suggest { text, .. } => Some(text),
            WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
        }
    }
}
