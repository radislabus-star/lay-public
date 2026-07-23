use super::atoms::{encode_wave_surface, physical_key_sequence, AtomChannel};
use super::compiler::{compile, compile_with_policy, ForwardPostingPolicy, TrainingWord};
use super::corruption::split_damages;
use super::format;
use super::runtime::{GrokkingCandidate, LexicalGrokkingMemory, ReadoutMode};

fn sequence_coherence(observed: &[u32], expected: &[u32]) -> u16 {
    super::runtime::sequence_coherence_milli(observed, expected)
}

fn legacy_sequence_coherence(observed: &[u32], expected: &[u32]) -> u16 {
    super::runtime::legacy_sequence_coherence_milli(observed, expected)
}

#[test]
fn atom_geometry_counts_substitution_insertion_and_transposition() {
    assert_eq!(super::runtime::damerau_distance(&[1, 2, 3], &[1, 2, 3]), 0);
    assert_eq!(super::runtime::damerau_distance(&[1, 9, 3], &[1, 2, 3]), 1);
    assert_eq!(
        super::runtime::damerau_distance(&[1, 2, 2, 3], &[1, 2, 3]),
        1
    );
    assert_eq!(super::runtime::damerau_distance(&[1, 3, 2], &[1, 2, 3]), 1);
}

#[test]
fn reconstruction_geometry_detects_deletions_and_deletion_transposition() {
    assert_ne!(
        super::runtime::reconstruction_modes(&[1, 4], &[1, 2, 3, 4])
            & super::runtime::RECONSTRUCTION_MODE_DELETION,
        0
    );
    assert_ne!(
        super::runtime::reconstruction_modes(&[1, 3, 2], &[1, 2, 3, 4])
            & super::runtime::RECONSTRUCTION_MODE_DELETION_TRANSPOSITION,
        0
    );
    assert_eq!(
        super::runtime::reconstruction_modes(&[1, 2, 4], &[1, 2, 3, 4]),
        0
    );
    assert_eq!(
        super::runtime::reconstruction_modes(&[9, 8, 7], &[1, 2, 3, 4]),
        0
    );
}

fn fixture_words() -> Vec<TrainingWord> {
    [
        "время",
        "работает",
        "восстановить",
        "переподключаю",
        "эффективная",
        "интеллекта",
        "предложение",
        "архитектура",
        "проверка",
        "корпус",
        "download",
        "wechat",
        "сугроб",
    ]
    .into_iter()
    .enumerate()
    .map(|(terminal_id, word)| {
        let (training, _) = split_damages(word);
        TrainingWord {
            terminal_id: terminal_id as u32,
            surface: word.to_string(),
            training_surfaces: training.into_iter().map(|item| item.surface).collect(),
        }
    })
    .collect()
}

#[test]
fn precise_channels_outweigh_candidate_birth_bigrams() {
    let atoms = encode_wave_surface("время");
    let precise = atoms
        .iter()
        .find(|atom| atom.key.channel == AtomChannel::CharacterGram)
        .copied()
        .expect("character trigram");
    let broad = atoms
        .iter()
        .find(|atom| atom.key.channel == AtomChannel::CharacterBigram)
        .copied()
        .expect("character bigram");
    assert!(precise.weight > broad.weight);
}

#[test]
fn encoder_emits_bounded_typed_channels() {
    let atoms = encode_wave_surface("переподключаю");
    assert!(atoms.len() <= 288);
    for channel in [
        AtomChannel::ByteGram,
        AtomChannel::CharacterGram,
        AtomChannel::CharacterBigram,
        AtomChannel::KeyboardGram,
        AtomChannel::KeyboardBigram,
        AtomChannel::CharacterBagGram,
        AtomChannel::KeyboardBagGram,
        AtomChannel::CharacterSkipGram,
        AtomChannel::KeyboardSkipGram,
        AtomChannel::CharacterAnchor,
        AtomChannel::BoundaryPosition,
    ] {
        assert!(atoms.iter().any(|atom| atom.key.channel == channel));
    }
}

#[test]
fn package_roundtrip_is_deterministic_and_stringless() {
    let words = fixture_words();
    let package = compile(&words).expect("compile fixture package");
    let first = format::encode(&package).expect("encode fixture package");
    let repeated = format::encode(&compile(&words).expect("repeat fixture compilation"))
        .expect("encode repeated fixture package");
    let decoded = format::decode(&first).expect("decode fixture package");
    let second = format::encode(&decoded).expect("re-encode fixture package");
    assert_eq!(first, second);
    assert_eq!(first, repeated);
    assert!(!first
        .windows("переподключаю".len())
        .any(|part| part == "переподключаю".as_bytes()));
}

#[test]
fn l11_profiles_are_bounded_and_cover_every_primary_center() {
    let package = compile(&fixture_words()).expect("compile fixture package");
    assert_eq!(package.center_phase_profiles.len(), package.centers.len());
    assert!(package
        .center_phase_profiles
        .iter()
        .all(|profile| profile.positive_count > 0
            && profile.positive_count <= 4
            && profile.anti_count <= 4
            && profile.hard_negative_count <= 2
            && profile.keyboard_geometry_count > 0
            && profile.keyboard_geometry_count <= 32));
}

#[test]
fn v4_package_remains_readable_with_permissive_l11_defaults() {
    let words = fixture_words();
    let mut package = compile(&words).expect("compile fixture package");
    package.center_phase_profiles.clear();
    package.positive_subcenters.clear();
    package.anti_subcenters.clear();
    package.hard_negative_subcenters.clear();
    package.keyboard_geometry_units.clear();
    package.restoration_calibration = super::restoration::RestorationCalibration::LEGACY_PERMISSIVE;
    let bytes = format::encode(&package).expect("encode legacy-compatible package");
    let decoded = format::decode(&bytes).expect("decode legacy-compatible package");
    assert!(decoded.center_phase_profiles.is_empty());
    assert_eq!(
        decoded.restoration_calibration,
        super::restoration::RestorationCalibration::LEGACY_PERMISSIVE
    );
}

#[test]
fn compressed_forward_format_preserves_exact_readout() {
    let words = fixture_words();
    let package = compile(&words).expect("compile fixture package");
    let original = LexicalGrokkingMemory {
        package: package.clone(),
    };
    let bytes = format::encode(&package).expect("encode compressed package");
    let compressed = LexicalGrokkingMemory::from_bytes(&bytes).expect("load compressed package");
    for word in &words {
        assert_eq!(
            original.readout(&word.surface, 64, ReadoutMode::Full),
            compressed.readout(&word.surface, 64, ReadoutMode::Full)
        );
        let (_, heldout) = split_damages(&word.surface);
        for damage in heldout.into_iter().take(3) {
            assert_eq!(
                original.readout(&damage.surface, 64, ReadoutMode::Full),
                compressed.readout(&damage.surface, 64, ReadoutMode::Full)
            );
        }
    }
}

#[test]
fn complete_posting_ablation_reports_baseline_saturation_without_dropping_links() {
    let words = fixture_words();
    let baseline = compile_with_policy(&words, ForwardPostingPolicy::BaselineBounded)
        .expect("compile bounded fixture");
    let complete = compile_with_policy(&words, ForwardPostingPolicy::Complete)
        .expect("compile complete fixture");
    assert_eq!(complete.diagnostics.forward_relations_dropped, 0);
    assert_eq!(
        baseline.diagnostics.forward_relations_before_policy,
        complete.diagnostics.forward_relations_before_policy
    );
    assert!(complete.package.forward_couplings.len() >= baseline.package.forward_couplings.len());
}

#[test]
fn heldout_damage_uses_phase_centers() {
    let words = fixture_words();
    let package = compile(&words).expect("compile fixture package");
    let bytes = format::encode(&package).expect("encode fixture package");
    let memory = LexicalGrokkingMemory::from_bytes(&bytes).expect("load fixture package");
    let (_, heldout) = split_damages("переподключаю");
    assert!(!heldout.is_empty());
    assert!(heldout.iter().any(|example| {
        memory
            .readout(&example.surface, 8, ReadoutMode::Full)
            .iter()
            .any(|candidate| candidate.terminal_id == 3)
    }));
}

#[test]
fn keyboard_channel_bridges_layout_surfaces() {
    let words = fixture_words();
    let package = compile(&words).expect("compile fixture package");
    let bytes = format::encode(&package).expect("encode fixture package");
    let memory = LexicalGrokkingMemory::from_bytes(&bytes).expect("load fixture package");
    let candidates = memory.readout("вщцтдщфв", 8, ReadoutMode::Full);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.terminal_id == 10));
    assert_ne!(
        package.centers[0].flags & super::model::CENTER_FLAG_CYRILLIC_SCRIPT,
        0
    );
    assert_ne!(
        package.centers[10].flags & super::model::CENTER_FLAG_ASCII_SCRIPT,
        0
    );
    let clean_keyboard = encode_wave_surface("download")
        .into_iter()
        .filter(|atom| atom.key.channel == AtomChannel::KeyboardGram)
        .map(|atom| (atom.key, atom.position))
        .collect::<Vec<_>>();
    let projected_keyboard = encode_wave_surface("вщцтдщфв")
        .into_iter()
        .filter(|atom| atom.key.channel == AtomChannel::KeyboardGram)
        .map(|atom| (atom.key, atom.position))
        .collect::<Vec<_>>();
    assert_eq!(clean_keyboard, projected_keyboard);
    let profile = memory.package.center_phase_profiles[10];
    let geometry = &memory.package.keyboard_geometry_units[profile.keyboard_geometry_start as usize
        ..profile.keyboard_geometry_start as usize + profile.keyboard_geometry_count as usize];
    assert_eq!(
        geometry,
        physical_key_sequence("download"),
        "physical keyboard geometry sequence was not retained"
    );
    assert_ne!(
        profile.flags & super::model::CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
        0
    );
    let mut candidates = memory.readout("вщцтдщфв", 64, ReadoutMode::Full);
    let restoration = memory.classify_restoration(
        "вщцтдщфв",
        &mut candidates,
        memory.package.restoration_calibration,
    );
    assert!(
        matches!(
            &restoration,
            super::restoration::RestorationReadout::Winner {
                candidate: super::restoration::RestorationCandidate {
                    terminal_id: 10,
                    ..
                }
            }
        ),
        "unexpected layout restoration: {restoration:?}"
    );
}

#[test]
fn physical_keyboard_geometry_preserves_layout_punctuation_keys() {
    let clean = physical_key_sequence("сугроб");
    let projected = physical_key_sequence("ceuhj,");
    assert_eq!(clean, projected);
    assert_eq!(clean.len(), 6);

    let package = compile(&fixture_words()).expect("compile fixture package");
    let memory = LexicalGrokkingMemory { package };
    let mut candidates = memory.readout("ceuhj,", 64, ReadoutMode::Full);
    let restoration = memory.classify_restoration(
        "ceuhj,",
        &mut candidates,
        memory.package.restoration_calibration,
    );
    assert_eq!(
        candidates
            .iter()
            .find(|candidate| candidate.terminal_id == 12)
            .map(|candidate| candidate.geometry_distance),
        Some(0)
    );
    assert!(
        !matches!(
            restoration,
            super::restoration::RestorationReadout::Abstain {
                reason: super::restoration::AbstainReason::OutsideCalibratedBasin,
                ..
            }
        ),
        "physical layout basin was lost: {restoration:?}"
    );
}

#[test]
fn backward_reconstruction_contains_only_clean_reference_atoms() {
    let words = fixture_words();
    let package = compile(&words).expect("compile fixture package");
    for word in &words {
        let clean_atoms = encode_wave_surface(&word.surface)
            .into_iter()
            .filter_map(|atom| package.graph.atom_id(atom.key))
            .collect::<std::collections::BTreeSet<_>>();
        let center = package.centers[word.terminal_id as usize];
        let start = center.coupling_start as usize;
        let end = start + center.coupling_count as usize;
        assert!(package.reverse_couplings[start..end]
            .iter()
            .all(|coupling| clean_atoms.contains(&coupling.peer_id)));
    }
}

#[test]
fn shadow_query_reads_terminal_ids_without_a_string_table() {
    let package = compile(&fixture_words()).expect("compile fixture package");
    let bytes = format::encode(&package).expect("encode fixture package");
    let path =
        std::env::temp_dir().join(format!("lay-l1-grokking-query-{}.bin", std::process::id()));
    std::fs::write(&path, bytes).expect("write fixture package");
    let report = super::runtime::query_package(&path, "врмея", 8).expect("query fixture package");
    assert!(report["candidates"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    assert_eq!(report["candidates"][0]["surface"], "время");
    let _ = std::fs::remove_file(path);
}

#[test]
fn sparse_omission_ambiguity_follows_surviving_symbol_order() {
    assert!(super::proof::is_ordered_subsequence(
        &"вкючть".chars().collect::<Vec<_>>(),
        &"включить".chars().collect::<Vec<_>>()
    ));
    assert!(super::proof::is_ordered_subsequence(
        &"вкючть".chars().collect::<Vec<_>>(),
        &"включать".chars().collect::<Vec<_>>()
    ));
    assert!(!super::proof::is_ordered_subsequence(
        &"вкючть".chars().collect::<Vec<_>>(),
        &"включен".chars().collect::<Vec<_>>()
    ));
}

#[test]
fn sequence_interference_preserves_exact_and_omission_reconstruction() {
    assert_eq!(sequence_coherence(&[1, 2, 3, 4], &[1, 2, 3, 4]), 1_000);
    assert_eq!(sequence_coherence(&[1, 3, 4], &[1, 2, 3, 4]), 1_000);
}

#[test]
fn legacy_sequence_contract_is_omission_only() {
    assert_eq!(legacy_sequence_coherence(&[1, 3, 4], &[1, 2, 3, 4]), 1_000);
    assert_eq!(legacy_sequence_coherence(&[1, 3, 2, 4], &[1, 2, 3, 4]), 750);
    assert_eq!(legacy_sequence_coherence(&[1, 2, 2, 3], &[1, 2, 3]), 750);
}

#[test]
fn sequence_interference_recognizes_repeated_fragments_and_transposition_mass() {
    assert_eq!(sequence_coherence(&[1, 2, 2, 3], &[1, 2, 3]), 1_000);
    assert!(sequence_coherence(&[1, 3, 2, 4], &[1, 2, 3, 4]) >= 700);
}

#[test]
fn sequence_interference_rejects_unrelated_reconstruction() {
    let related = sequence_coherence(&[1, 3, 2, 4], &[1, 2, 3, 4]);
    let unrelated = sequence_coherence(&[5, 6, 7, 8], &[1, 2, 3, 4]);
    assert!(related > unrelated);
    assert_eq!(unrelated, 750);
}

#[test]
fn exact_legacy_sequence_certificate_vetoes_weaker_partial_waves() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            legacy_sequence_milli: 1_000,
            sequence_milli: 1_000,
            legacy_settled_energy: 1_000,
            settled_energy: 1_000,
            length_relation: 1,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            legacy_sequence_milli: 750,
            sequence_milli: 1_000,
            legacy_settled_energy: 900,
            settled_energy: 1_650,
            length_relation: 0,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 3,
            legacy_sequence_milli: 750,
            sequence_milli: 900,
            legacy_settled_energy: 800,
            settled_energy: 1_250,
            length_relation: 1,
            ..GrokkingCandidate::default()
        },
    ];
    super::runtime::apply_sequence_certificate_interference(&mut candidates, ReadoutMode::Full);
    assert_eq!(candidates[1].settled_energy, 900);
    assert_eq!(candidates[1].sequence_milli, 750);
    assert_eq!(candidates[2].settled_energy, 800);
    assert_eq!(candidates[2].sequence_milli, 750);
}

#[test]
fn partial_sequence_waves_remain_free_without_an_exact_certificate() {
    let mut candidates = [GrokkingCandidate {
        legacy_sequence_milli: 750,
        sequence_milli: 900,
        legacy_settled_energy: 800,
        settled_energy: 1_250,
        ..GrokkingCandidate::default()
    }];
    super::runtime::apply_sequence_certificate_interference(&mut candidates, ReadoutMode::Full);
    assert_eq!(candidates[0].settled_energy, 1_250);
    assert_eq!(candidates[0].sequence_milli, 900);
}

#[test]
fn weaker_exact_sequence_cannot_veto_the_winner_owned_partial_wave() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            legacy_sequence_milli: 1_000,
            sequence_milli: 1_000,
            legacy_settled_energy: 800,
            settled_energy: 800,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            legacy_sequence_milli: 750,
            sequence_milli: 900,
            legacy_settled_energy: 900,
            settled_energy: 1_350,
            ..GrokkingCandidate::default()
        },
    ];

    super::runtime::apply_sequence_certificate_interference(&mut candidates, ReadoutMode::Full);

    assert_eq!(candidates[1].sequence_milli, 900);
    assert_eq!(candidates[1].settled_energy, 1_350);
}

#[test]
fn position_certificate_cannot_cross_a_length_changing_basin() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            position_milli: 0,
            sequence_milli: 1_000,
            length_relation: 1,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            position_milli: 1_000,
            length_relation: 0,
            ..GrokkingCandidate::default()
        },
    ];

    super::runtime::apply_position_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, 1);
}

#[test]
fn position_certificate_can_resolve_an_uncertified_length_incumbent() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            position_milli: 0,
            sequence_milli: 750,
            length_relation: 1,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            position_milli: 800,
            sequence_milli: 750,
            length_relation: 0,
            ..GrokkingCandidate::default()
        },
    ];

    super::runtime::apply_position_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, 2);
}

#[test]
fn position_certificate_respects_cross_length_energy_lease() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            sequence_milli: 750,
            settled_energy: 2_000,
            length_relation: 1,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            position_milli: 800,
            sequence_milli: 750,
            settled_energy: 1_000,
            length_relation: 0,
            ..GrokkingCandidate::default()
        },
    ];

    super::runtime::apply_position_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, 1);
}

#[test]
fn equal_length_position_certificate_requires_stronger_sequence() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            sequence_milli: 750,
            settled_energy: 1_000,
            length_relation: 0,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            position_milli: 800,
            sequence_milli: 750,
            settled_energy: 700,
            length_relation: 0,
            ..GrokkingCandidate::default()
        },
    ];

    super::runtime::apply_position_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, 1);
}
