use super::atoms::{encode_wave_surface, physical_key_sequence, AtomChannel};
use super::compiler::{
    compile, compile_training_corpus_with_policy, compile_with_policy, ForwardPostingPolicy,
    TrainingWord,
};
use super::corruption::split_damages;
use super::format;
use super::runtime::{GrokkingCandidate, LexicalGrokkingMemory, ReadoutMode};
use super::training_corpus::TrainingCorpus;

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
        super::runtime::reconstruction_modes(&[1, 2, 3], &[1, 2, 3, 4])
            & super::runtime::RECONSTRUCTION_MODE_SUFFIX_TRUNCATION,
        0
    );
    assert_ne!(
        super::runtime::reconstruction_modes(&[2, 3, 4], &[1, 2, 3, 4])
            & super::runtime::RECONSTRUCTION_MODE_PREFIX_TRUNCATION,
        0
    );
    assert_ne!(
        super::runtime::reconstruction_modes(&[1, 2, 4], &[1, 2, 3, 4])
            & super::runtime::RECONSTRUCTION_MODE_SINGLE_DELETION,
        0
    );
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
        super::runtime::reconstruction_modes(&[9, 8, 7], &[1, 2, 3, 4]),
        0
    );
}

#[test]
fn surface_operator_reconstruction_preserves_direction_and_distance() {
    assert_eq!(
        super::runtime::surface_operator_reconstruction_modes("amooe", "alone"),
        super::runtime::RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION
    );
    assert_eq!(
        super::runtime::surface_operator_reconstruction_modes("amooe", "among"),
        0
    );
    assert_eq!(
        super::runtime::surface_operator_reconstruction_modes("всемя", "время"),
        super::runtime::RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION
    );
    assert_eq!(
        super::runtime::surface_operator_reconstruction_modes("abedc", "abcde"),
        super::runtime::RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION
    );
    assert_eq!(
        super::runtime::surface_operator_reconstruction_modes("abdce", "abcde"),
        super::runtime::RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION
    );
}

#[test]
fn alphabet_damage_rotation_is_reversible() {
    for ch in ['а', 'я', 'ё', 'a', 'z', 'А', 'Z'] {
        let damaged =
            crate::nanda_wave::surface_damage::alphabet_successor(ch).expect("alphabet member");
        assert_eq!(
            crate::nanda_wave::surface_damage::alphabet_predecessor(damaged),
            Some(ch)
        );
    }
}

#[test]
fn sampled_ambiguity_indexes_cross_operator_sources() {
    let words = ["абе", "абазе", "абаке", "care", "core", "crew"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let heldout = [
        (
            1,
            super::corruption::DamageExample {
                class: "omission_transposition",
                surface: "аабе".to_string(),
            },
        ),
        (
            3,
            super::corruption::DamageExample {
                class: "missing_letter",
                surface: "cre".to_string(),
            },
        ),
    ];
    let mut ambiguity = std::collections::HashMap::new();
    super::proof::populate_sampled_ambiguity(&words, &heldout, &mut ambiguity);
    assert!(ambiguity["аабе"].contains(&0));
    assert!(ambiguity["cre"].contains(&5));
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
fn packed_training_arena_preserves_surfaces_and_exact_ambiguity() {
    let words = vec![
        TrainingWord {
            terminal_id: 0,
            surface: "alpha".to_string(),
            training_surfaces: vec!["shared".to_string(), "cross".to_string()],
        },
        TrainingWord {
            terminal_id: 1,
            surface: "beta".to_string(),
            training_surfaces: vec!["shared".to_string(), "private".to_string()],
        },
        TrainingWord {
            terminal_id: 2,
            surface: "gamma".to_string(),
            training_surfaces: vec!["cross".to_string()],
        },
    ];
    let corpus = TrainingCorpus::from_words(&words).expect("pack training corpus");
    assert_eq!(corpus.training_surface_count(), 5);
    assert_eq!(
        corpus.word_surfaces(&corpus.words()[0]).collect::<Vec<_>>(),
        vec!["alpha", "shared", "cross"]
    );
    let owners = corpus.ambiguous_surface_owners();
    assert_eq!(owners["shared"], vec![0, 1]);
    assert_eq!(owners["cross"], vec![0, 2]);
    assert!(!owners.contains_key("private"));
}

#[test]
fn packed_training_shards_merge_in_terminal_order_without_surface_drift() {
    let first = TrainingCorpus::from_words(&[TrainingWord {
        terminal_id: 0,
        surface: "alpha".to_string(),
        training_surfaces: vec!["alhpa".to_string()],
    }])
    .expect("pack first shard");
    let second = TrainingCorpus::from_words(&[TrainingWord {
        terminal_id: 1,
        surface: "beta".to_string(),
        training_surfaces: vec!["btea".to_string(), "bea".to_string()],
    }])
    .expect("pack second shard");

    let mut merged = TrainingCorpus::try_with_capacity(2, 3, 13).expect("allocate merged corpus");
    merged.append_shard(first).expect("append first shard");
    merged.append_shard(second).expect("append second shard");

    assert_eq!(
        merged.word_surfaces(&merged.words()[0]).collect::<Vec<_>>(),
        vec!["alpha", "alhpa"]
    );
    assert_eq!(
        merged.word_surfaces(&merged.words()[1]).collect::<Vec<_>>(),
        vec!["beta", "btea", "bea"]
    );
}

#[test]
fn packed_training_compiler_is_byte_deterministic() {
    let words = fixture_words();
    let corpus = TrainingCorpus::from_words(&words).expect("pack training corpus");
    let first = compile_training_corpus_with_policy(&corpus, ForwardPostingPolicy::Complete)
        .expect("compile packed corpus");
    let second = compile_training_corpus_with_policy(&corpus, ForwardPostingPolicy::Complete)
        .expect("compile packed corpus again");
    assert_eq!(
        format::encode(&first.package).expect("encode first package"),
        format::encode(&second.package).expect("encode second package")
    );
}

#[test]
fn damaged_evidence_cannot_change_the_primary_crystal() {
    let clean_words = fixture_words()
        .into_iter()
        .map(|mut word| {
            word.training_surfaces.clear();
            word
        })
        .collect::<Vec<_>>();
    let damaged_words = fixture_words();
    let clean = compile(&clean_words).expect("compile clean primary crystal");
    let damaged = compile(&damaged_words).expect("compile residual evidence crystal");

    assert_eq!(clean.graph, damaged.graph);
    assert_eq!(clean.atoms, damaged.atoms);
    assert_eq!(clean.forward_couplings, damaged.forward_couplings);
    assert_eq!(clean.reverse_couplings, damaged.reverse_couplings);
    assert_eq!(clean.decoder_nodes, damaged.decoder_nodes);
    assert_eq!(clean.centers.len(), damaged.centers.len());
    for (clean_center, damaged_center) in clean.centers.iter().zip(&damaged.centers) {
        assert_eq!(clean_center.wave_code, damaged_center.wave_code);
        assert_eq!(clean_center.coupling_start, damaged_center.coupling_start);
        assert_eq!(clean_center.coupling_count, damaged_center.coupling_count);
        assert_eq!(clean_center.crystal_support, damaged_center.crystal_support);
        assert_eq!(clean_center.stability, damaged_center.stability);
        assert_eq!(
            clean_center.decoder_terminal,
            damaged_center.decoder_terminal
        );
        assert_eq!(clean_center.surface_len, damaged_center.surface_len);
        assert_eq!(clean_center.flags, damaged_center.flags);
    }
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
fn v6_roundtrip_preserves_scale_atom_posting_count() {
    let words = fixture_words();
    let mut package = compile(&words).expect("compile fixture package");
    for atom in &mut package.atoms {
        atom.coupling_start = 0;
        atom.coupling_count = 0;
    }
    let scale_count = u32::from(u16::MAX) + 1;
    let relation = super::model::WaveCoupling {
        peer_id: 0,
        strength: 200,
        phase_relation: 0,
        position_mode: 128,
        flags: 0,
    };
    package.forward_couplings = vec![relation; scale_count as usize];
    package.atoms[0].coupling_count = scale_count;

    let first = format::encode(&package).expect("encode v6 scale package");
    assert_eq!(&first[..8], b"LAYL1C06");
    let decoded = format::decode(&first).expect("decode v6 scale package");
    assert_eq!(decoded.atoms[0].coupling_count, scale_count);
    assert_eq!(decoded.forward_couplings.len(), scale_count as usize);
    assert_eq!(
        format::encode(&decoded).expect("re-encode v6 scale package"),
        first
    );
}

#[test]
fn v5_atom_layout_migrates_to_deterministic_v6() {
    let package = compile(&fixture_words()).expect("compile fixture package");
    let v5 = format::encode_v5_compat(&package).expect("encode v5 compatibility fixture");
    assert_eq!(&v5[..8], b"LAYL1C05");
    let migrated = format::decode(&v5).expect("decode v5 compatibility fixture");
    let v6 = format::encode(&migrated).expect("migrate v5 fixture to v6");
    assert_eq!(&v6[..8], b"LAYL1C06");
    assert_eq!(
        format::encode(&format::decode(&v6).expect("decode migrated v6"))
            .expect("re-encode migrated v6"),
        v6
    );
}

#[test]
fn v6_decoder_rejects_impossible_scale_count_before_allocation() {
    let package = compile(&fixture_words()).expect("compile fixture package");
    let mut bytes = format::encode(&package).expect("encode v6 fixture");
    let atom_offset = u64::from_le_bytes(
        bytes[104..112]
            .try_into()
            .expect("atom offset header field"),
    ) as usize;
    for index in 0..package.atoms.len() {
        let count_offset = atom_offset + index * 28 + 20;
        bytes[count_offset..count_offset + 4].copy_from_slice(&0_u32.to_le_bytes());
    }
    bytes[56..60].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[atom_offset + 20..atom_offset + 24].copy_from_slice(&u32::MAX.to_le_bytes());
    format::refresh_checksum(&mut bytes);

    let error = format::decode(&bytes).expect_err("reject impossible compressed count");
    assert!(error.contains("count exceeds section capacity"));
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
            && profile.ambiguity_count <= 8
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
    package.ambiguity_subcenters.clear();
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
    let original = LexicalGrokkingMemory::from_package(package.clone());
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
fn fused_proof_readouts_match_independent_modes_exactly() {
    let package = compile(&fixture_words()).expect("compile fixture package");
    let memory = LexicalGrokkingMemory::from_package(package);
    let modes = [
        ReadoutMode::Full,
        ReadoutMode::WithoutPhase,
        ReadoutMode::WithoutAnti,
        ReadoutMode::WithoutSequence,
        ReadoutMode::LegacySequence,
        ReadoutMode::WithoutSequenceCertificate,
        ReadoutMode::WithoutPairwise,
        ReadoutMode::WithoutPosition,
    ];

    for surface in ["врем", "переподкючаю", "вщцтдщфв", "downlaod"] {
        let fused = memory.readout_modes(surface, 64, &modes);
        let independent = modes
            .iter()
            .copied()
            .map(|mode| memory.readout(surface, 64, mode))
            .collect::<Vec<_>>();
        assert_eq!(fused, independent, "fused readout drifted for {surface}");
    }
}

#[test]
fn exact_singleton_fast_path_preserves_the_full_readout_winner() {
    let words = fixture_words();
    let package = compile(&words).expect("compile fixture package");
    let memory = LexicalGrokkingMemory::from_package(package);

    for word in &words {
        let singleton = memory.readout(&word.surface, 1, ReadoutMode::Full);
        let full = memory.readout(&word.surface, 64, ReadoutMode::Full);
        assert_eq!(
            singleton.first().map(|candidate| candidate.terminal_id),
            full.first().map(|candidate| candidate.terminal_id),
            "exact singleton winner drifted for {}",
            word.surface
        );
        assert!(
            singleton
                .first()
                .is_some_and(|candidate| candidate.exact_reconstruction),
            "exact singleton lost its no-op certificate for {}",
            word.surface
        );
    }
}

#[test]
fn exact_singleton_fast_path_skips_filtered_reverse_atoms() {
    let words = fixture_words();
    let mut package = compile(&words).expect("compile fixture package");
    let target = 0_usize;
    let center = package.centers[target];
    let start = center.coupling_start as usize;
    let end = start + center.coupling_count as usize;
    let filtered = package.reverse_couplings[start..end]
        .iter_mut()
        .find(|coupling| coupling.flags == 0)
        .expect("fixture word must have a lexical reverse coupling");
    filtered.flags = 2;
    let memory = LexicalGrokkingMemory::from_package(package);

    let singleton = memory.readout(&words[target].surface, 1, ReadoutMode::Full);
    let full = memory.readout(&words[target].surface, 64, ReadoutMode::Full);
    assert_eq!(
        singleton.first().map(|candidate| candidate.terminal_id),
        full.first().map(|candidate| candidate.terminal_id),
    );
    assert_eq!(
        singleton.first().map(|candidate| candidate.terminal_id),
        Some(target as u32),
    );
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
    let memory = LexicalGrokkingMemory::from_package(package);
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
fn raw_layout_projection_preserves_leading_punctuation_keys() {
    let words = ["бикс", "brc", "brcc"]
        .into_iter()
        .enumerate()
        .map(|(terminal_id, word)| TrainingWord {
            terminal_id: terminal_id as u32,
            surface: word.to_string(),
            training_surfaces: Vec::new(),
        })
        .collect::<Vec<_>>();
    let package = compile(&words).expect("compile leading layout punctuation fixture");
    let memory = LexicalGrokkingMemory::from_package(package);
    let candidates = memory.readout(",brc", 64, ReadoutMode::Full);
    assert_eq!(
        candidates
            .iter()
            .find(|candidate| candidate.terminal_id == 0)
            .map(|candidate| candidate.geometry_distance),
        Some(0),
        "the comma key must remain available as the Russian letter б"
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
fn append_only_manifest_adds_centers_and_applies_tombstones_without_rewriting_base() {
    let root = std::env::temp_dir().join(format!(
        "lay-l11-composite-{}-{}",
        std::process::id(),
        crate::stable_hash::mix64_golden(0x11c0_ffee)
    ));
    std::fs::create_dir_all(&root).expect("create composite fixture root");
    let base_source_path = root.join("base.v7.bin");
    let base_path = root.join("base.v8.bin");
    let delta_path = root.join("delta.bin");
    let receipt_path = root.join("proof.json");
    let failed_receipt_path = root.join("failed-proof.json");
    let manifest_path = root.join("runtime.json");

    let mut base_words = fixture_words();
    for word in &mut base_words {
        word.training_surfaces.clear();
    }
    std::fs::write(
        &base_source_path,
        format::encode_compact_depth0(&compile(&base_words).expect("compile base"))
            .expect("encode compact base"),
    )
    .expect("write compact base");
    super::v8::build_lazy_v8_package(&base_source_path, &base_path).expect("build V8 base");
    let (training, _) = split_damages("кристаллизатор");
    let delta_words = vec![TrainingWord {
        terminal_id: 0,
        surface: "кристаллизатор".to_string(),
        training_surfaces: training.into_iter().map(|item| item.surface).collect(),
    }];
    std::fs::write(
        &delta_path,
        format::encode(&compile(&delta_words).expect("compile delta")).expect("encode delta"),
    )
    .expect("write delta");
    std::fs::write(&receipt_path, b"{\"verdict\":\"PASS\"}\n").expect("write proof receipt");
    std::fs::write(&failed_receipt_path, b"{\"verdict\":\"FAIL\"}\n")
        .expect("write failed proof receipt");

    super::composite::initialize_manifest(&manifest_path, &base_path).expect("initialize manifest");
    let empty_composite =
        super::runtime::L1RestorationHost::load(&manifest_path).expect("load empty composite");
    assert_eq!(
        empty_composite.restore("время", 8)["result"]["authority"],
        false
    );
    assert_eq!(
        empty_composite.restore("время", 8)["result"]["verdict"],
        "lattice"
    );
    assert!(super::composite::admit_delta(
        &manifest_path,
        &delta_path,
        &failed_receipt_path,
        Some("rejected fixture"),
    )
    .is_err());
    super::composite::admit_delta(&manifest_path, &delta_path, &receipt_path, Some("fixture"))
        .expect("admit delta");
    let host = super::runtime::L1RestorationHost::load(&manifest_path).expect("load composite");
    assert_eq!(host.terminal_count(), base_words.len() as u32 + 1);
    assert_eq!(
        host.terminal_for_exact_surface("кристаллизатор"),
        Some(base_words.len() as u32)
    );
    assert_eq!(
        host.restore("кристаллизатор", 8)["result"]["authority"],
        false
    );
    assert_eq!(
        host.lattice_seed_rows("время", 8)[0].1,
        "время",
        "an exact base surface must remain the first composite seed"
    );
    assert_eq!(
        host.lattice_seed_rows("кристаллизатор", 8)[0].1,
        "кристаллизатор",
        "an exact delta surface must remain the first composite seed"
    );
    let benchmark = super::runtime::benchmark_package(&manifest_path, "кристаллизатор", 3, 8)
        .expect("benchmark composite");
    assert_eq!(benchmark["delta_count"], 1);
    assert_eq!(benchmark["manifest_generation"], 2);

    super::composite::admit_tombstone(&manifest_path, "время", &receipt_path, Some("fixture"))
        .expect("admit tombstone");
    assert_eq!(
        super::composite::manifest_generation(&manifest_path).expect("read generation"),
        Some(3)
    );
    let host = super::runtime::L1RestorationHost::load(&manifest_path)
        .expect("reload composite with tombstone");
    assert_eq!(host.terminal_for_exact_surface("время"), None);
    assert_eq!(host.decode_terminal(0), None);
    assert!(host
        .lattice_seed_rows("врмея", 8)
        .iter()
        .all(|(_, surface, _)| surface != "время"));
    assert_eq!(host.stats().delta_count, 1);
    assert_eq!(host.stats().tombstone_count, 1);

    let first_manifest = manifest_path.clone();
    let first_receipt = receipt_path.clone();
    let first = std::thread::spawn(move || {
        super::composite::admit_tombstone(
            &first_manifest,
            "когда",
            &first_receipt,
            Some("concurrent fixture"),
        )
    });
    let second_manifest = manifest_path.clone();
    let second_receipt = receipt_path.clone();
    let second = std::thread::spawn(move || {
        super::composite::admit_tombstone(
            &second_manifest,
            "куда",
            &second_receipt,
            Some("concurrent fixture"),
        )
    });
    first
        .join()
        .expect("first admission thread")
        .expect("first admission");
    second
        .join()
        .expect("second admission thread")
        .expect("second admission");
    assert_eq!(
        super::composite::manifest_generation(&manifest_path).expect("read final generation"),
        Some(5)
    );
    let host = super::runtime::L1RestorationHost::load(&manifest_path)
        .expect("reload composite after concurrent admissions");
    assert_eq!(host.stats().tombstone_count, 3);

    let _ = std::fs::remove_dir_all(root);
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

#[test]
fn learned_ambiguity_relation_links_only_a_nearby_competitor_basin() {
    assert!(super::runtime::ambiguity_geometry_link(1, 2, 3));
    assert!(super::runtime::ambiguity_geometry_link(2, 1, 3));
    assert!(!super::runtime::ambiguity_geometry_link(1, 3, 3));
    assert!(!super::runtime::ambiguity_geometry_link(2, 4, 3));
}

#[test]
fn exact_surface_center_owns_clean_readout_before_energy() {
    let mut candidates = [
        GrokkingCandidate {
            terminal_id: 1,
            settled_energy: 2_000,
            ..GrokkingCandidate::default()
        },
        GrokkingCandidate {
            terminal_id: 2,
            settled_energy: 1_000,
            exact_reconstruction: true,
            ..GrokkingCandidate::default()
        },
    ];

    candidates.sort_unstable_by(super::runtime::candidate_order);

    assert_eq!(candidates[0].terminal_id, 2);
}
