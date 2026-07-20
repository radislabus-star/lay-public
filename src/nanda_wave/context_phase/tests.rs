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
        canonical_scene_wave(&hashes, ContextPhaseMode::Full, |_| None)
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

    let generic_only = package.raw_readout(&vector, &vector, token, ContextPhaseMode::Full);
    assert_eq!(generic_only.anti_micro, 0);
    assert!(generic_only.margin_micro > 0);

    package.profiles[0]
        .hard_negative
        .push(PhaseCenter::from_center(vector.clone(), 1));
    let false_winner = package.raw_readout(&vector, &vector, token, ContextPhaseMode::Full);
    assert!(false_winner.anti_micro > 0);
    assert_eq!(
        package
            .raw_readout(&vector, &vector, token, ContextPhaseMode::NoAnti)
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
    );
    let no_signature = package.raw_readout(
        &signature_vector,
        &signature_vector,
        token,
        ContextPhaseMode::NoSignatureProfile,
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
    );
    assert!(!signature_only.profile_present);
    assert_eq!(
        signature_only.disposition,
        ContextPhaseDisposition::Unavailable
    );
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
        context_preserving_candidate_token(&context, "у нас есть"),
        Some("есть".to_string())
    );
    assert_eq!(
        context_preserving_candidate_token(&context, "есть"),
        Some("есть".to_string())
    );
    assert_eq!(
        context_preserving_candidate_token(&context, "ун ас есть"),
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
    let budget = if cfg!(debug_assertions) { 1_000 } else { 250 };
    assert!(p99 <= budget, "L3 hot readout p99={p99}us > {budget}us");
}

#[test]
fn tracked_context_phase_exposes_case_competition_for_intellect_scene() {
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
    assert_eq!(readouts[2].disposition, ContextPhaseDisposition::Support);
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
