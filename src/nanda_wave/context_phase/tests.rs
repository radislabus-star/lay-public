use super::*;
use std::time::Instant;

#[test]
fn semantic_relation_is_an_additive_channel_and_never_erases_surface() {
    assert_eq!(semantic_relation_weights(1), (1.0, 0.0));
    assert_eq!(semantic_relation_weights(2), (1.0, 0.80));
    assert_eq!(semantic_relation_weights(16), (1.0, 0.80));
    assert_eq!(candidate_semantic_relation_weight(1), 0.0);
    assert_eq!(candidate_semantic_relation_weight(2), 0.85);
}

#[test]
fn canonical_scene_wave_matches_hot_readout_without_semantic_anchors() {
    let context = super::super::llmwave::tokenize("на улице идет дождь");
    let hashes = context
        .iter()
        .map(|token| hash_text(token))
        .collect::<Vec<_>>();
    let package = ContextPhasePackage::default();

    assert_eq!(
        package.context_vector(&context, ContextPhaseMode::Full),
        canonical_scene_wave(&hashes, ContextPhaseMode::Full, |_, _| None)
    );
}

#[test]
fn relation_role_schema_expands_tokens_without_changing_legacy_scene_atoms() {
    let context = vec!["Apple".to_string(), "оплата".to_string()];
    let legacy = context_atom_hashes(&context, SIGNATURE_SCHEMA_MORPHOLOGY_PHASE);
    let relation = context_atom_hashes(&context, SIGNATURE_SCHEMA_RELATION_ROLES);

    assert_eq!(
        legacy,
        vec![context_exact_hash("Apple"), context_exact_hash("оплата")]
    );
    assert_eq!(relation.len(), legacy.len() * 2);
    assert_eq!(relation[0], context_exact_hash("Apple"));
    assert_eq!(relation[1], context_role_hash("Apple"));
    assert_ne!(context_role_hash("Apple"), context_role_hash("wave"));
    assert_ne!(context_role_hash("и"), context_role_hash("слово"));
}

#[test]
fn learned_relation_role_transfers_conjunction_support_to_unseen_entity() {
    let teacher_context = vec!["оплата".to_string(), "Samsung".to_string()];
    let runtime_context = vec!["оплата".to_string(), "Apple".to_string()];
    let role_hash = context_role_hash("Samsung");
    let mut package = ContextPhasePackage {
        signature_schema: SIGNATURE_SCHEMA_RELATION_ROLES,
        global_threshold_micro: 1,
        competition_threshold_micro: 1,
        semantic_states: vec![
            TokenSemanticState {
                token_hash: context_exact_hash("оплата"),
                support: 8,
                center: phase_center_from_sum(&empty_vector(CELLS)),
            },
            TokenSemanticState {
                token_hash: role_hash,
                support: 8,
                center: phase_center_from_sum(&empty_vector(CELLS)),
            },
        ],
        ..ContextPhasePackage::default()
    };
    package
        .semantic_states
        .sort_by_key(|state| state.token_hash);
    let teacher_scene =
        package.candidate_relation_vector(&teacher_context, "и", ContextPhaseMode::Full);
    package.profiles.push(ContextCandidateProfile {
        token_hash: hash_text("и"),
        positive_examples: 8,
        negative_examples: 0,
        threshold_micro: 1,
        positive: vec![PhaseCenter::from_center(teacher_scene, 8)],
        negative: Vec::new(),
        hard_negative: Vec::new(),
    });

    let readout = package.score_candidates(&runtime_context, &["и"])[0];

    assert_eq!(context_role_hash("Samsung"), context_role_hash("Apple"));
    assert_eq!(readout.context_known_tokens, 2);
    assert_eq!(readout.disposition, ContextPhaseDisposition::Support);
}

#[test]
fn legacy_schema_does_not_gain_one_token_context_authority() {
    let context = vec!["контекст".to_string()];
    let mut package = ContextPhasePackage {
        signature_schema: SIGNATURE_SCHEMA_MORPHOLOGY_PHASE,
        global_threshold_micro: 1,
        competition_threshold_micro: 1,
        semantic_states: vec![TokenSemanticState {
            token_hash: context_exact_hash("контекст"),
            support: 8,
            center: phase_center_from_sum(&empty_vector(CELLS)),
        }],
        ..ContextPhasePackage::default()
    };
    let scene = package.candidate_relation_vector(&context, "и", ContextPhaseMode::Full);
    package.profiles.push(ContextCandidateProfile {
        token_hash: hash_text("и"),
        positive_examples: 8,
        negative_examples: 0,
        threshold_micro: 1,
        positive: vec![PhaseCenter::from_center(scene, 8)],
        negative: Vec::new(),
        hard_negative: Vec::new(),
    });

    let readout = package.score_candidates(&context, &["и"])[0];

    assert_eq!(readout.context_known_tokens, 1);
    assert_eq!(readout.disposition, ContextPhaseDisposition::Neutral);
}

#[test]
fn quiet_learned_basin_blocks_unary_neighbor_authority() {
    let context = vec!["контекст".to_string()];
    let mut package = ContextPhasePackage {
        signature_schema: SIGNATURE_SCHEMA_RELATION_ROLES,
        global_threshold_micro: 1,
        competition_threshold_micro: 1,
        semantic_states: vec![TokenSemanticState {
            token_hash: context_exact_hash("контекст"),
            support: 8,
            center: phase_center_from_sum(&empty_vector(CELLS)),
        }],
        ..ContextPhasePackage::default()
    };
    let winner = "девочка";
    let quiet = "девчонка";
    let winner_vector = package.candidate_relation_vector(&context, winner, ContextPhaseMode::Full);
    package.profiles = vec![
        ContextCandidateProfile {
            token_hash: hash_text(winner),
            positive_examples: 8,
            negative_examples: 0,
            threshold_micro: 1,
            positive: vec![PhaseCenter::from_center(winner_vector, 8)],
            negative: Vec::new(),
            hard_negative: Vec::new(),
        },
        ContextCandidateProfile {
            token_hash: hash_text(quiet),
            positive_examples: 8,
            negative_examples: 0,
            threshold_micro: 1,
            positive: vec![PhaseCenter::from_center(empty_vector(CELLS), 8)],
            negative: Vec::new(),
            hard_negative: Vec::new(),
        },
    ];
    package.profiles.sort_by_key(|profile| profile.token_hash);

    let readouts = package.score_candidates(&context, &[winner, quiet, "неизвестный"]);

    assert!(readouts[0].margin_micro > readouts[1].margin_micro);
    assert_eq!(readouts[1].positive_micro, 0);
    assert_eq!(readouts[0].disposition, ContextPhaseDisposition::Neutral);
}

#[test]
fn relation_role_alone_cannot_authorize_a_one_token_scene() {
    let entities = [
        "Samsung", "Google", "Huawei", "Xiaomi", "Mozilla", "Amazon", "Lenovo", "Nokia", "Toyota",
        "Canon", "Spotify", "Netflix", "Adobe", "Oracle", "Siemens", "Philips",
    ];
    let mut corpus = String::new();
    for (index, entity) in entities.iter().enumerate() {
        corpus.push_str(&format!("{entity} и Partner{index}\n"));
    }
    for label in [
        "wave", "lane", "mode", "group", "phase", "class", "slot", "field",
    ] {
        corpus.push_str(&format!("{label} b signal\n"));
    }
    corpus.push_str("GitHub b branch\nGitLab b branch\n");
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: &corpus,
        max_fragments: 0,
        min_profile_support: 2,
    });

    let entity = package.score_candidates(&["Apple".to_string()], &["и", "b"]);
    let plain = package.score_candidates(&["wave".to_string()], &["и", "b"]);
    let technical = package.score_candidates(&["GitHub".to_string()], &["и", "b"]);
    let technical_buffer = package.score_candidates(&["buffer".to_string()], &["и", "b"]);

    assert_eq!(package.signature_schema, SIGNATURE_SCHEMA_RELATION_ROLES);
    assert_ne!(
        entity[0].disposition,
        ContextPhaseDisposition::Support,
        "{entity:?}"
    );
    assert_ne!(
        plain[0].disposition,
        ContextPhaseDisposition::Support,
        "{plain:?}"
    );
    assert_ne!(
        technical[0].disposition,
        ContextPhaseDisposition::Support,
        "{technical:?}"
    );
    assert!(
        technical_buffer
            .iter()
            .all(|readout| readout.disposition != ContextPhaseDisposition::Support),
        "{technical_buffer:?}"
    );
}

#[test]
fn balanced_exact_profiles_leave_a_one_token_scene_unresolved() {
    let mut corpus = String::new();
    for subject in [
        "фильм",
        "сериал",
        "ответ",
        "вариант",
        "результат",
        "подход",
        "проект",
        "пример",
    ] {
        corpus.push_str(&format!("Мне нравится {subject}\n"));
        corpus.push_str(&format!("Мне нравятся эти {subject}\n"));
    }
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: &corpus,
        max_fragments: 0,
        min_profile_support: 2,
    });

    let readouts = package.score_candidates(&["Мне".to_string()], &["нравится", "нравятся"]);

    assert!(
        readouts
            .iter()
            .all(|readout| readout.disposition != ContextPhaseDisposition::Support),
        "{readouts:?}"
    );
}

#[test]
fn relation_scene_uses_sentence_context_beyond_the_immediate_left_role() {
    let entities = [
        "Nimbus", "Atlas", "Orion", "Vega", "Sirius", "Nova", "Astra", "Lumen",
    ];
    let mut corpus = String::new();
    for entity in entities {
        corpus.push_str(&format!("покупатель выбрал {entity} и Partner\n"));
        corpus.push_str(&format!("заказчик сравнил {entity} и Partner\n"));
        corpus.push_str(&format!("compiler пометил {entity} b register\n"));
        corpus.push_str(&format!("debugger оставил {entity} b branch\n"));
    }
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: &corpus,
        max_fragments: 0,
        min_profile_support: 2,
    });

    let conjunction = package.score_candidates(
        &[
            "покупатель".to_string(),
            "выбрал".to_string(),
            "Quasar".to_string(),
        ],
        &["и", "b"],
    );
    let technical = package.score_candidates(
        &[
            "compiler".to_string(),
            "пометил".to_string(),
            "Quasar".to_string(),
        ],
        &["и", "b"],
    );

    assert_eq!(
        conjunction[0].disposition,
        ContextPhaseDisposition::Support,
        "{conjunction:?}"
    );
    assert_ne!(
        technical[0].disposition,
        ContextPhaseDisposition::Support,
        "{technical:?}"
    );
}

#[test]
fn pair_edge_is_canonical_and_order_independent() {
    let context = super::super::llmwave::tokenize("на улице идет");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let left = hash_text("дождь");
    let right = hash_text("день");
    let key = PairKey::new(left, right).unwrap();
    package.pair_profiles.push(ContextPairPhaseProfile {
        low_hash: key.low_hash,
        high_hash: key.high_hash,
        low_wins: vec![PhaseCenter::from_center(scene.clone(), 2)],
        high_wins: Vec::new(),
        hard_low_wins: Vec::new(),
        hard_high_wins: Vec::new(),
    });

    assert_eq!(
        package.pair_edge(&scene, left, right, 1, 2, true),
        PairEdgeOutcome::LowWins
    );
    assert_eq!(
        package.pair_edge(&scene, right, left, 2, 1, true),
        PairEdgeOutcome::LowWins
    );
}

#[test]
fn repeated_pair_direction_narrows_only_its_learned_uncertainty_band() {
    let threshold = 0.20;
    let sparse = directional_evidence_margin(threshold, 2, 2);
    let repeated = directional_evidence_margin(threshold, 2, 128);

    assert!(sparse > repeated);
    assert!(repeated > threshold);
}

#[test]
fn repeated_pair_direction_settles_across_independent_subcenters() {
    let mut first_scene = empty_vector(CELLS);
    first_scene[0] = PhaseCell { re: 1.0, im: 0.0 };
    let mut second_scene = empty_vector(CELLS);
    second_scene[1] = PhaseCell { re: 1.0, im: 0.0 };
    let mut unseen_scene = empty_vector(CELLS);
    unseen_scene[2] = PhaseCell { re: 1.0, im: 0.0 };
    let low = hash_text("первый");
    let high = hash_text("второй");
    let key = PairKey::new(low, high).unwrap();
    let view_index = sentence::PAIR_VIEW_LEFT_EXACT;
    let view_winner = pair_view_hash(key.low_hash, view_index);
    let view_loser = pair_view_hash(key.high_hash, view_index);
    let view_key = PairKey::new(view_winner, view_loser).unwrap();
    let centers = vec![
        PhaseCenter::from_center(first_scene.clone(), 1),
        PhaseCenter::from_center(second_scene.clone(), 1),
    ];
    let (low_wins, high_wins) = if view_winner == view_key.low_hash {
        (centers, Vec::new())
    } else {
        (Vec::new(), centers)
    };
    let package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        pair_profiles: vec![ContextPairPhaseProfile {
            low_hash: view_key.low_hash,
            high_hash: view_key.high_hash,
            low_wins,
            high_wins,
            ..ContextPairPhaseProfile::default()
        }],
        ..ContextPhasePackage::default()
    };

    assert_eq!(
        package
            .pair_view_edge_evidence(
                &first_scene,
                key.low_hash,
                key.high_hash,
                1,
                2,
                view_index,
                false,
            )
            .outcome,
        PairEdgeOutcome::LowWins
    );
    assert_eq!(
        package
            .pair_view_edge_evidence(
                &second_scene,
                key.low_hash,
                key.high_hash,
                1,
                2,
                view_index,
                false,
            )
            .outcome,
        PairEdgeOutcome::LowWins
    );
    assert_eq!(
        package
            .pair_view_edge_evidence(
                &unseen_scene,
                key.low_hash,
                key.high_hash,
                1,
                2,
                view_index,
                false,
            )
            .outcome,
        PairEdgeOutcome::Unknown
    );
}

#[test]
fn relation_pair_key_is_canonical_without_retaining_candidate_text() {
    let left = hash_text("дождь");
    let right = hash_text("день");
    assert_eq!(
        PairKey::relation(left, 17, right, 29),
        PairKey::relation(right, 29, left, 17),
    );
    assert!(PairKey::relation(left, 17, right, 29).is_some_and(PairKey::is_relation));
}

#[test]
fn generalized_l2_pair_fills_unknown_exact_pair_but_cannot_create_support() {
    let context = super::super::llmwave::tokenize("на улице идет");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let low = hash_text("дождь");
    let high = hash_text("день");
    let key = PairKey::relation(low, 17, high, 29).unwrap();
    let mut profile = ContextPairPhaseProfile {
        low_hash: key.low_hash,
        high_hash: key.high_hash,
        ..ContextPairPhaseProfile::default()
    };
    if 17 < 29 {
        profile
            .low_wins
            .push(PhaseCenter::from_center(scene.clone(), 2));
    } else {
        profile
            .high_wins
            .push(PhaseCenter::from_center(scene.clone(), 2));
    }
    package.pair_profiles.push(profile);

    let outcome = package.pair_edge(&scene, low, high, 17, 29, true);
    assert_eq!(
        outcome,
        if low < high {
            PairEdgeOutcome::LowWins
        } else {
            PairEdgeOutcome::HighWins
        }
    );
}

#[test]
fn generalized_relation_preserves_signature_winner_when_lexical_hash_order_flips() {
    let context = super::super::llmwave::tokenize("сильная контекстная сцена");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let first = hash_text("первый");
    let second = hash_text("второй");
    let (left, right) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    // `right` owns the low L2 signature even though its lexical hash is high.
    let key = PairKey::relation(left, 29, right, 17).unwrap();
    package.pair_profiles.push(ContextPairPhaseProfile {
        low_hash: key.low_hash,
        high_hash: key.high_hash,
        low_wins: vec![PhaseCenter::from_center(scene.clone(), 2)],
        ..ContextPairPhaseProfile::default()
    });

    // Low signature wins, therefore lexical high (`right`) must win.
    assert_eq!(
        package.pair_edge(&scene, left, right, 29, 17, false),
        PairEdgeOutcome::HighWins
    );
}

fn pair_profile_for_scene(scene: &[PhaseCell], winner: u64, loser: u64) -> ContextPairPhaseProfile {
    let key = PairKey::new(winner, loser).unwrap();
    let center = PhaseCenter::from_center(scene.to_vec(), 2);
    let mut profile = ContextPairPhaseProfile {
        low_hash: key.low_hash,
        high_hash: key.high_hash,
        ..ContextPairPhaseProfile::default()
    };
    if winner == key.low_hash {
        profile.low_wins.push(center);
    } else {
        profile.high_wins.push(center);
    }
    profile
}

#[test]
fn pairwise_tie_is_neutral_not_a_hidden_veto() {
    let context = super::super::llmwave::tokenize("на улице идет");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let left = hash_text("дождь");
    let right = hash_text("день");
    let key = PairKey::new(left, right).unwrap();
    package.pair_profiles.push(ContextPairPhaseProfile {
        low_hash: key.low_hash,
        high_hash: key.high_hash,
        low_wins: vec![PhaseCenter::from_center(scene.clone(), 2)],
        high_wins: vec![PhaseCenter::from_center(scene.clone(), 2)],
        ..ContextPairPhaseProfile::default()
    });

    let dominance = package.pairwise_dominance(
        &scene,
        &[(left, (0, 1)), (right, (0, 2))],
        ContextPhaseMode::Full,
    );
    assert!(!dominance.blocks(left));
    assert!(!dominance.blocks(right));
}

#[test]
fn pairwise_cycle_leaves_every_cycle_member_neutral() {
    let context = super::super::llmwave::tokenize("на улице идет");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let first = hash_text("дождь");
    let second = hash_text("день");
    let third = hash_text("свет");
    package.pair_profiles = vec![
        pair_profile_for_scene(&scene, first, second),
        pair_profile_for_scene(&scene, second, third),
        pair_profile_for_scene(&scene, third, first),
    ];
    package
        .pair_profiles
        .sort_by_key(|profile| (profile.low_hash, profile.high_hash));

    let dominance = package.pairwise_dominance(
        &scene,
        &[(first, (0, 1)), (second, (0, 2)), (third, (0, 3))],
        ContextPhaseMode::Full,
    );
    assert!(dominance.blocks(first));
    assert!(dominance.blocks(second));
    assert!(dominance.blocks(third));
}

#[test]
fn pairwise_dominance_does_not_depend_on_lattice_order() {
    let context = super::super::llmwave::tokenize("на улице идет");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let winner = hash_text("дождь");
    let loser = hash_text("день");
    package.pair_profiles = vec![pair_profile_for_scene(&scene, winner, loser)];

    let forward = package.pairwise_dominance(
        &scene,
        &[(winner, (0, 1)), (loser, (0, 2))],
        ContextPhaseMode::Full,
    );
    let reverse = package.pairwise_dominance(
        &scene,
        &[(loser, (0, 2)), (winner, (0, 1))],
        ContextPhaseMode::Full,
    );
    assert_eq!(forward.blocks(winner), reverse.blocks(winner));
    assert_eq!(forward.blocks(loser), reverse.blocks(loser));
    assert!(!forward.blocks(winner));
    assert!(forward.blocks(loser));
}

#[test]
fn repeated_hard_pairwise_bank_suppresses_but_never_creates_unary_support() {
    let context = super::super::llmwave::tokenize("на улице идет");
    let mut package = ContextPhasePackage {
        pairwise_threshold_micro: 10,
        ..ContextPhasePackage::default()
    };
    let scene = package.context_vector(&context, ContextPhaseMode::Full);
    let low = hash_text("дождь");
    let high = hash_text("день");
    let key = PairKey::new(low, high).unwrap();
    package.pair_profiles = vec![ContextPairPhaseProfile {
        low_hash: key.low_hash,
        high_hash: key.high_hash,
        hard_low_wins: vec![PhaseCenter::from_center(scene.clone(), 2)],
        ..ContextPairPhaseProfile::default()
    }];

    let with_hard = package.pairwise_dominance(
        &scene,
        &[(low, (0, 1)), (high, (0, 2))],
        ContextPhaseMode::Full,
    );
    let without_hard = package.pairwise_dominance(
        &scene,
        &[(low, (0, 1)), (high, (0, 2))],
        ContextPhaseMode::NoHardPairwise,
    );
    assert!(with_hard.blocks(key.high_hash));
    assert!(!without_hard.blocks(key.low_hash));
    assert!(!without_hard.blocks(key.high_hash));
}

#[test]
fn shard_merge_reclusters_pair_banks_under_their_budget() {
    let low = hash_text("дождь");
    let high = hash_text("день");
    let shards = (0..(MAX_PAIR_CENTERS_PER_BANK + 3))
        .map(|mode| {
            let mut center = vec![PhaseCell::default(); CELLS];
            center[mode % CELLS] = PhaseCell { re: 1.0, im: 0.0 };
            ContextPhasePackage {
                pair_profiles: vec![ContextPairPhaseProfile {
                    low_hash: low.min(high),
                    high_hash: low.max(high),
                    low_wins: vec![PhaseCenter::from_center(center, 2)],
                    ..ContextPairPhaseProfile::default()
                }],
                ..ContextPhasePackage::default()
            }
        })
        .collect();

    let merged = ContextPhasePackage::merge_shards(shards);
    assert_eq!(merged.pair_profiles.len(), 1);
    assert!(merged.pair_profiles[0].low_wins.len() <= MAX_PAIR_CENTERS_PER_BANK);
}

#[test]
fn surface_consensus_merge_keeps_only_cross_surface_profiles_and_pairs() {
    let shared = hash_text("общий");
    let one_surface = hash_text("одноразовый");
    let low = hash_text("дождь").min(hash_text("день"));
    let high = hash_text("дождь").max(hash_text("день"));
    let profile = |token_hash| ContextCandidateProfile {
        token_hash,
        positive_examples: 2,
        negative_examples: 0,
        threshold_micro: 0,
        positive: Vec::new(),
        negative: Vec::new(),
        hard_negative: Vec::new(),
    };
    let pair = || ContextPairPhaseProfile {
        low_hash: low,
        high_hash: high,
        low_wins: vec![PhaseCenter::from_center(
            vec![PhaseCell::default(); CELLS],
            2,
        )],
        ..ContextPairPhaseProfile::default()
    };
    let shards = vec![
        ContextPhasePackage {
            profiles: vec![profile(shared), profile(one_surface)],
            pair_profiles: vec![pair()],
            ..ContextPhasePackage::default()
        },
        ContextPhasePackage {
            profiles: vec![profile(shared)],
            pair_profiles: vec![pair()],
            ..ContextPhasePackage::default()
        },
        ContextPhasePackage::default(),
    ];

    let (merged, report) = ContextPhasePackage::merge_shards_with_min_surface_support(shards, 2);

    assert_eq!(report.surface_count, 3);
    assert_eq!(report.min_surface_support, 2);
    assert_eq!(report.profiles_before_consensus, 2);
    assert_eq!(report.profiles_after_consensus, 1);
    assert_eq!(report.pairs_before_consensus, 1);
    assert_eq!(report.pairs_after_consensus, 1);
    assert_eq!(merged.profiles.len(), 1);
    assert_eq!(merged.profiles[0].token_hash, shared);
    assert_eq!(merged.pair_profiles.len(), 1);
}

#[test]
fn dominated_negative_margin_candidate_cannot_reenter_as_synthetic_zero() {
    let candidates = ["цель", "ложный"];
    let mut dominance = PairwiseDominance::default();
    dominance.losses.insert(candidate_token_hash("ложный"));
    let readouts = [
        ContextPhaseReadout {
            profile_present: true,
            positive_examples: 2,
            margin_micro: -3,
            ..ContextPhaseReadout::default()
        },
        ContextPhaseReadout {
            profile_present: true,
            positive_examples: 2,
            margin_micro: -4,
            ..ContextPhaseReadout::default()
        },
    ];

    assert_eq!(
        survivor_ranking(&candidates, &readouts, &dominance),
        vec![(0, -3)]
    );
}

#[test]
fn generic_competitor_negative_cannot_become_a_unary_veto() {
    let token = "кандидат";
    let token_hash = hash_text(token);
    let mut vector = vec![PhaseCell::default(); CELLS];
    vector[0] = PhaseCell { re: 1.0, im: 0.0 };
    let mut package = ContextPhasePackage {
        profiles: vec![ContextCandidateProfile {
            token_hash,
            positive_examples: 2,
            negative_examples: 2,
            threshold_micro: 0,
            positive: vec![PhaseCenter::from_center(vector.clone(), 2)],
            negative: vec![PhaseCenter::from_center(vector.clone(), 2)],
            hard_negative: Vec::new(),
        }],
        ..ContextPhasePackage::default()
    };

    let generic_only = package.raw_readout(&vector, &vector, token, ContextPhaseMode::Full, false);
    assert_eq!(generic_only.anti_micro, 0);
    assert!(generic_only.margin_micro > 0);

    package.profiles[0]
        .hard_negative
        .push(PhaseCenter::from_center(vector.clone(), 1));
    let false_winner = package.raw_readout(&vector, &vector, token, ContextPhaseMode::Full, false);
    assert!(false_winner.anti_micro > 0);
    assert_eq!(
        package
            .raw_readout(&vector, &vector, token, ContextPhaseMode::NoAnti, false)
            .anti_micro,
        0
    );
}

#[test]
fn signature_profile_strengthens_exact_profile_without_becoming_authority() {
    let token = "дождь";
    let token_hash = hash_text(token);
    let signature = candidate_l2_signature(token);
    let exact_vector = vec![PhaseCell { re: 1.0, im: 0.0 }; CELLS];
    let signature_vector = vec![PhaseCell { re: 0.0, im: 1.0 }; CELLS];
    let package = ContextPhasePackage {
        profiles: vec![ContextCandidateProfile {
            token_hash,
            positive_examples: 2,
            negative_examples: 0,
            threshold_micro: 0,
            positive: vec![PhaseCenter::from_center(exact_vector, 2)],
            negative: Vec::new(),
            hard_negative: Vec::new(),
        }],
        signature_profiles: vec![ContextCandidateProfile {
            token_hash: signature,
            positive_examples: 2,
            negative_examples: 0,
            threshold_micro: 0,
            positive: vec![PhaseCenter::from_center(signature_vector.clone(), 2)],
            negative: Vec::new(),
            hard_negative: Vec::new(),
        }],
        ..ContextPhasePackage::default()
    };

    let full = package.raw_readout(
        &signature_vector,
        &signature_vector,
        token,
        ContextPhaseMode::Full,
        false,
    );
    let no_signature = package.raw_readout(
        &signature_vector,
        &signature_vector,
        token,
        ContextPhaseMode::NoSignatureProfile,
        false,
    );
    assert!(full.signature_profile_present);
    assert!(full.margin_micro > no_signature.margin_micro);

    let signature_only = ContextPhasePackage {
        signature_profiles: package.signature_profiles.clone(),
        ..ContextPhasePackage::default()
    }
    .raw_readout(
        &signature_vector,
        &signature_vector,
        token,
        ContextPhaseMode::Full,
        false,
    );
    assert!(!signature_only.profile_present);
    assert_eq!(signature_only.disposition, ContextPhaseDisposition::Neutral);
    assert!(signature_only.signature_profile_present);
    assert!(signature_only.positive_micro > 0);
    assert!(signature_only.margin_micro > 0);
}

#[test]
fn learned_context_phase_separates_same_surface_family_by_scene() {
    let corpus = concat!(
        "на улице утром идет дождь. на улице вечером идет дождь. ",
        "в комнате утром горит свет. в комнате вечером горит свет. ",
        "сегодня на улице идет дождь. сегодня в комнате горит свет."
    );
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: corpus,
        max_fragments: 0,
        min_profile_support: 2,
    });
    let context = super::super::llmwave::tokenize("сегодня на улице идет");
    let readouts = package.score_candidates(&context, &["дождь", "свет"]);

    assert!(readouts[0].profile_present);
    assert!(readouts[0].margin_micro > readouts[1].margin_micro);
}

#[test]
fn no_phase_ablation_removes_context_authority() {
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text:
            "на улице опять идет дождь. вечером на улице идет дождь. утром на улице идет дождь.",
        max_fragments: 0,
        min_profile_support: 2,
    });
    let context = super::super::llmwave::tokenize("вечером на улице идет");
    let readouts = package.score_candidates_with_mode(
        &context,
        &["дождь", "домик"],
        ContextPhaseMode::NoPhase,
    );

    assert!(readouts
        .iter()
        .all(|readout| readout.disposition == ContextPhaseDisposition::Unavailable));
}

#[test]
fn duplicate_sources_do_not_compete_with_the_same_lexical_center() {
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: concat!(
            "на улице опять идет дождь. ",
            "вечером на улице идет дождь. ",
            "утром на улице идет дождь. ",
            "в комнате вечером горит свет."
        ),
        max_fragments: 0,
        min_profile_support: 2,
    });
    let context = super::super::llmwave::tokenize("вечером на улице идет");
    let readouts = package.score_candidates(&context, &["дождь", "дождь", "домик"]);

    assert_eq!(readouts[0].margin_micro, readouts[1].margin_micro);
    assert_eq!(
        readouts[0].competition_margin_micro,
        readouts[1].competition_margin_micro
    );
    assert!(readouts[0].competition_margin_micro > 0);
}

#[test]
fn next_token_context_rejects_candidates_that_rewrite_the_left_context() {
    let context = super::super::llmwave::tokenize("у нас");

    assert_eq!(
        context_preserving_candidate_token(&context, "у нас есть", false),
        Some("есть".to_string())
    );
    assert_eq!(
        context_preserving_candidate_token(&context, "есть", false),
        Some("есть".to_string())
    );
    assert_eq!(
        context_preserving_candidate_token(&context, "ун ас есть", false),
        None
    );
}

#[test]
fn relation_schema_preserves_mixed_case_left_context_at_runtime_boundary() {
    let original = tokenize_context_text("Нужно посмотреть через MTC можно оплатить Apple b");
    let context = &original[..original.len() - 1];

    assert_eq!(
        context_preserving_candidate_token(
            context,
            "Нужно посмотреть через MTC можно оплатить Apple и",
            true,
        ),
        Some("и".to_string())
    );
    assert_eq!(
        context_preserving_candidate_token(
            context,
            "нужно посмотреть через MTC можно оплатить Apple и",
            true,
        ),
        None
    );
}

#[test]
fn compiled_hot_context_readout_stays_inside_microsecond_budget() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");
    let context = super::super::llmwave::tokenize("на улице снова идет");
    let candidates = ["дождь", "день", "дом"];
    let _ = package.score_candidates(&context, &candidates);
    let mut elapsed = Vec::with_capacity(1_200);
    for _ in 0..1_200 {
        let started = Instant::now();
        let _ = package.score_candidates(&context, &candidates);
        elapsed.push(started.elapsed().as_micros());
    }
    elapsed.sort_unstable();
    let p99 = elapsed[elapsed.len() * 99 / 100];
    let max = *elapsed.last().unwrap_or(&0);
    eprintln!("l3 context phase hot readout: p99={p99}us max={max}us");
    // Debug instrumentation is intentionally noisy; the release branch is the
    // runtime latency contract.
    let budget = if cfg!(debug_assertions) { 5_000 } else { 250 };
    assert!(p99 <= budget, "L3 hot readout p99={p99}us > {budget}us");
}

#[test]
fn compiled_sentence_context_readout_stays_inside_preedit_budget() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");
    let original = "нужно проверить редкий всплеск при построении с";
    let replacements = [
        "нужно проверить редкий всплеск при построении сразу",
        "нужно проверить редкий всплеск при построении слова",
        "нужно проверить редкий всплеск при построении связи",
        "нужно проверить редкий всплеск при построении сцены",
        "нужно проверить редкий всплеск при построении сигнала",
        "нужно проверить редкий всплеск при построении системы",
        "нужно проверить редкий всплеск при построении списка",
        "нужно проверить редкий всплеск при построении сети",
        "нужно проверить редкий всплеск при построении слоя",
        "нужно проверить редкий всплеск при построении состояния",
        "нужно проверить редкий всплеск при построении структуры",
        "нужно проверить редкий всплеск при построении строки",
    ];
    let _ = readout_candidates_with_package(&package, original, &replacements);
    let mut elapsed = Vec::with_capacity(1_200);
    for _ in 0..1_200 {
        let started = Instant::now();
        let _ = readout_candidates_with_package(&package, original, &replacements);
        elapsed.push(started.elapsed().as_micros());
    }
    elapsed.sort_unstable();
    let p50 = elapsed[elapsed.len() / 2];
    let p99 = elapsed[elapsed.len() * 99 / 100];
    let max = *elapsed.last().unwrap_or(&0);
    eprintln!("l3 sentence hot readout: p50={p50}us p99={p99}us max={max}us");
    let budget = if cfg!(debug_assertions) {
        120_000
    } else {
        5_000
    };
    assert!(
        p99 <= budget,
        "L3 sentence hot readout p99={p99}us > {budget}us"
    );
}

#[test]
fn tracked_context_phase_exposes_case_competition_but_abstains_on_unknown_candidates() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");
    let context = super::super::llmwave::tokenize("ты записал нашу новую концепцию");
    let candidates = ["интелект", "интеллект", "интеллекта", "интерес"];
    let readouts = package.score_candidates(&context, &candidates);

    for (candidate, readout) in candidates.iter().zip(&readouts) {
        eprintln!("candidate={candidate} readout={readout:?}");
    }
    assert!(readouts[2].profile_present);
    assert!(readouts[2].semantic_support > 0);
    assert!(readouts[2].competition_margin_micro != 0);
    assert!(readouts.iter().any(|readout| !readout.profile_present));
    assert!(readouts
        .iter()
        .all(|readout| readout.disposition != ContextPhaseDisposition::Support));
}

#[test]
fn tracked_package_declares_relation_role_signature_projection() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");

    assert_eq!(package.signature_schema, SIGNATURE_SCHEMA_RELATION_ROLES);
}

#[test]
fn tracked_relation_package_uses_sentence_context_without_single_token_authority() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");
    let candidates = ["и", "b"];
    let readouts = |context: &str| {
        let tokens = super::tokenize_context_text(context);
        package.score_candidates(&tokens, &candidates)
    };
    let comparison_scene = readouts("Покупатель сравнил Quasar");
    let payment_scene = readouts("Нужно посмотреть через MTC можно оплатить Apple");
    let technical_scene = readouts("compiler сохранил Quasar");
    let isolated_entity = readouts("Apple");
    let plain_label = readouts("wave");
    let technical_label = readouts("GitHub");

    assert_eq!(package.signature_schema, SIGNATURE_SCHEMA_RELATION_ROLES);
    assert_ne!(
        comparison_scene[1].disposition,
        ContextPhaseDisposition::Support
    );
    assert_eq!(
        comparison_scene[0].disposition,
        ContextPhaseDisposition::Support
    );
    assert_eq!(
        payment_scene[0].disposition,
        ContextPhaseDisposition::Neutral,
        "{payment_scene:?}"
    );
    assert!(payment_scene[0].margin_micro > payment_scene[0].threshold_micro);
    assert_eq!(payment_scene[0].pairwise_unknown_edges, 1);
    assert_ne!(
        technical_scene[0].disposition,
        ContextPhaseDisposition::Support
    );
    assert_ne!(
        isolated_entity[0].disposition,
        ContextPhaseDisposition::Support
    );
    assert_ne!(plain_label[0].disposition, ContextPhaseDisposition::Support);
    assert_ne!(
        technical_label[0].disposition,
        ContextPhaseDisposition::Support
    );
}

#[test]
fn sparse_context_profile_cannot_override_established_center_without_bayes_margin() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");
    let context = super::super::llmwave::tokenize("на улице снова начался");
    let candidates = ["дождь", "ложь", "дочь", "дрожь"];
    let readouts = package.score_candidates(&context, &candidates);

    assert_ne!(readouts[3].disposition, ContextPhaseDisposition::Support);
}
