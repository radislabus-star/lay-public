use super::super::compact_format::encode_package as encode_compact_package;
use super::super::compiler::compile_l2_package;
use super::super::format::encode_package;
use super::super::teacher::L2TeacherCorpus;
use super::*;
use sha2::Digest;

fn productive_birth(
    surface: &str,
    lemma_id: u32,
    lemma_evidence: u16,
    geometry_evidence: u16,
) -> ProductiveL2FormBirth {
    ProductiveL2FormBirth {
        surface: surface.to_string(),
        lemma_id,
        source_form_ref: 0,
        source_feature_mask: 0,
        target_feature_mask: 0,
        geometry_evidence_milli: geometry_evidence,
        profile_evidence_milli: 1_000,
        slot_evidence_milli: 1_000,
        context_positive_support: 1,
        context_unlabeled_alternative_support: 0,
        context_posterior_milli: 1_000,
        context_observed: true,
        context_pair_evidence: Vec::new(),
        joint_evidence_milli: lemma_evidence.min(geometry_evidence),
        positive_support: 1,
        anti_support: 0,
        family_specificity: 1,
        lemma_atom_evidence_milli: lemma_evidence,
        lemma_wave_distance: 0,
        exact_surface_form_ref: None,
        status: ProductiveBirthStatus::ShadowUnverified,
    }
}

#[test]
fn productive_readout_cannot_collapse_distinct_lemma_basins() {
    let births = vec![
        productive_birth("strong", 1, 900, 900),
        productive_birth("retained", 2, 500, 500),
    ];

    assert_eq!(
        productive_l2_readout("damaged", &births),
        ProductiveL2Readout::Tied {
            surfaces: vec!["retained".to_string(), "strong".to_string()]
        }
    );
}

#[test]
fn productive_readout_can_settle_a_form_inside_one_lemma_basin() {
    let births = vec![
        productive_birth("strong", 1, 900, 900),
        productive_birth("weak", 1, 500, 500),
    ];

    assert_eq!(
        productive_l2_readout("damaged", &births),
        ProductiveL2Readout::Winner {
            surface: "strong".to_string()
        }
    );
}

#[test]
fn directional_context_pair_can_settle_only_its_same_lemma_competitor() {
    let mut preferred = productive_birth("preferred", 1, 700, 700);
    preferred.target_feature_mask = 11;
    preferred.context_pair_evidence = vec![ProductiveL2ContextPairEvidence {
        competitor_feature_mask: 22,
        evidence: ProductiveContextPairEvidence {
            positive_support: 3,
            anti_support: 1,
            posterior_milli: 666,
            context_observed: true,
            exact_positive_support: 3,
            exact_anti_support: 1,
            ..ProductiveContextPairEvidence::default()
        },
    }];
    let mut competitor = productive_birth("competitor", 1, 900, 900);
    competitor.target_feature_mask = 22;
    competitor.context_pair_evidence = vec![ProductiveL2ContextPairEvidence {
        competitor_feature_mask: 11,
        evidence: ProductiveContextPairEvidence {
            positive_support: 1,
            anti_support: 3,
            posterior_milli: 333,
            context_observed: true,
            exact_positive_support: 1,
            exact_anti_support: 3,
            ..ProductiveContextPairEvidence::default()
        },
    }];
    let mut other_lemma = productive_birth("other-lemma", 2, 500, 500);
    other_lemma.target_feature_mask = 22;

    assert_eq!(
        productive_l2_readout(
            "damaged",
            &[preferred.clone(), competitor, other_lemma.clone()]
        ),
        ProductiveL2Readout::Tied {
            surfaces: vec!["other-lemma".to_string(), "preferred".to_string()]
        }
    );
    assert_eq!(
        productive_l2_readout("damaged", &[preferred, other_lemma]),
        ProductiveL2Readout::Tied {
            surfaces: vec!["other-lemma".to_string(), "preferred".to_string()]
        }
    );
}

#[test]
fn productive_readout_abstains_when_input_is_already_a_generated_form() {
    let births = vec![
        productive_birth("observed", 1, 900, 900),
        productive_birth("alternative", 1, 500, 500),
    ];

    assert_eq!(
        productive_l2_readout("observed", &births),
        ProductiveL2Readout::Abstain
    );
}

#[test]
fn exact_geometry_prefers_a_unique_stronger_operator() {
    let preferred = preferred_exact_geometry_form_refs(
        "acbd",
        [(1, "abcd".to_string()), (2, "axbd".to_string())],
    );

    assert_eq!(preferred, BTreeSet::from([1]));
}

#[test]
fn exact_geometry_keeps_same_operator_ambiguity_tied() {
    let preferred = preferred_exact_geometry_form_refs(
        "abcd",
        [(1, "bacd".to_string()), (2, "acbd".to_string())],
    );

    assert_eq!(preferred, BTreeSet::from([1, 2]));
}

#[test]
fn exact_geometry_keeps_untyped_distance_ambiguity_tied() {
    let preferred = preferred_exact_geometry_form_refs(
        "abcd",
        [(1, "abed".to_string()), (2, "abfd".to_string())],
    );

    assert_eq!(preferred, BTreeSet::from([1, 2]));
}

#[test]
fn bounded_readout_reserves_grounded_l11_forms_before_compositional_fill() {
    let mut candidates = (0_u32..40)
        .map(|form_ref| L2LocalCandidate {
            form_ref,
            l1_terminal_id: Some(form_ref),
            surface: format!("surface-{form_ref}"),
            l1_evidence_milli: 1_000 - form_ref as i32,
            slot_phase_milli: 0,
            neighbor_pressure: 0,
            competition_pressure: 0,
            explicit_competition_pressure: 0,
            local_score: 1_000 - form_ref as i32,
            lemma_ids: Vec::new(),
            feature_masks: Vec::new(),
        })
        .collect::<Vec<_>>();
    let grounded = BTreeSet::from([39]);

    truncate_with_grounded_l11_reserve(&mut candidates, &grounded, 32);

    assert_eq!(candidates.len(), 32);
    assert!(candidates.iter().any(|candidate| candidate.form_ref == 39));
    assert!(!candidates.iter().any(|candidate| candidate.form_ref == 31));
    assert!(candidates
        .windows(2)
        .all(|pair| local_candidate_order(&pair[0], &pair[1]).is_le()));
}

#[test]
fn reference_and_compact_runtime_readouts_are_exactly_equal() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tдом\tдом\tnoun:nom:sg\n\
         F\tдом\tдома\tnoun:gen:sg\n\
         F\tдома\tдома\tnoun:nom:pl\n\
         F\tпосмотреть\tпосмотреть\tverb:inf:perf\n\
         F\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\n\
         F\tпросмотреть\tпросмотри\tverb:imp_excl:sg:imp:perf\n\
         T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
         T\tдом\tдома\tnoun:gen:sg\tнет _\n\
         T\tдома\tдома\tnoun:nom:pl\tработаю _\n\
         T\tпосмотреть\tпосмотреть\tverb:inf:perf\tхочу _\n\
         H\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\n\
         NT\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n\
         NH\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([
        ("дом", 7),
        ("дома", 11),
        ("посмотреть", 17),
        ("просмотри", 23),
    ]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let reference_bytes = encode_package(&package).expect("reference encode");
    let (compact_bytes, _) = encode_compact_package(&package).expect("compact encode");
    let compact_sha256: [u8; 32] = sha2::Sha256::digest(&compact_bytes).into();
    let reference = StandaloneL2Field::from_bytes(&reference_bytes).expect("reference load");
    let compact = StandaloneL2Field::from_bytes(&compact_bytes).expect("compact load");
    let mmap_path = std::env::temp_dir().join(format!(
        "lay-l2-compact-mmap-{}-{}.bin",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::write(&mmap_path, &compact_bytes).expect("write mmap fixture");
    let mapped = StandaloneL2Field::load(&mmap_path).expect("mmap load");
    std::fs::remove_file(&mmap_path).expect("remove mmap fixture");

    assert_eq!(reference.package_counts(), compact.package_counts());
    assert_eq!(reference.package_identity(), None);
    let compact_identity = compact.package_identity().expect("owned compact identity");
    assert_eq!(compact_identity.bytes(), compact_bytes.len() as u64);
    assert_eq!(compact_identity.sha256(), compact_sha256);
    let mapped_identity = mapped
        .package_identity()
        .expect("retained mmap compact identity after pathname deletion");
    assert_eq!(mapped_identity.bytes(), compact_bytes.len() as u64);
    assert_eq!(mapped_identity.sha256(), compact_sha256);
    assert_eq!(reference.package_storage().0, "reference_v2_owned");
    assert_eq!(
        compact.package_storage(),
        ("compact_v2_compositional", compact_bytes.len())
    );
    assert_eq!(
        compact.compositional_index_source(),
        "compact_v2_owned_view"
    );
    assert!(!compact.package_mmap_backed());
    assert!(compact.compositional_index_bytes() > 0);
    assert!(compact.compositional_index_bytes() < compact.compositional_index_view_bytes());
    assert!(compact.compositional_index_view_bytes() > 0);
    #[cfg(target_os = "linux")]
    {
        assert!(mapped.package_mmap_backed());
        assert_eq!(mapped.compositional_index_source(), "compact_v2_mmap_view");
    }
    assert_eq!(
        mapped.compositional_index_bytes(),
        compact.compositional_index_bytes()
    );
    assert_eq!(
        mapped.compositional_index_view_bytes(),
        compact.compositional_index_view_bytes()
    );
    for runtime in [&reference, &compact, &mapped] {
        for lemma_id in 0..runtime.package.lemma_centers().len() as u32 {
            let (primary_pos, fast) = package_canonical_source(&runtime.package, lemma_id)
                .expect("fast canonical source");
            let lemma = package_lemma(&runtime.package, lemma_id).expect("full lemma");
            let full = super::super::productive::canonical_source(&lemma.forms)
                .expect("full canonical source");
            assert_eq!(primary_pos, lemma.primary_pos);
            assert_eq!(&fast, full);
        }
    }
    assert_eq!(
        reference.single_edit_form_refs("дмо", 16),
        compact.single_edit_form_refs("дмо", 16)
    );

    let probes = [
        (
            "нет _",
            vec![L2LexicalSeed {
                terminal_id: Some(7),
                surface: None,
                evidence_milli: 900,
                origin: L2LexicalSeedOrigin::GroundedL11,
            }],
        ),
        (
            "_ сюда",
            vec![
                L2LexicalSeed {
                    terminal_id: Some(17),
                    surface: None,
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
                L2LexicalSeed {
                    terminal_id: Some(23),
                    surface: None,
                    evidence_milli: 960,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
            ],
        ),
        (
            "неизвестная сцена _",
            vec![
                L2LexicalSeed {
                    terminal_id: None,
                    surface: Some("дом".to_string()),
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
                L2LexicalSeed {
                    terminal_id: None,
                    surface: Some("дома".to_string()),
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
            ],
        ),
    ];
    for (context, seeds) in probes {
        assert_eq!(
            reference.readout(context, &seeds, 8),
            compact.readout(context, &seeds, 8),
            "runtime parity failed for context {context:?}"
        );
        assert_eq!(
            reference.readout(context, &seeds, 8),
            mapped.readout(context, &seeds, 8),
            "mmap runtime parity failed for context {context:?}"
        );
    }
}

#[test]
fn compact_runtime_indexes_preserve_terminal_form_and_lemma_bindings() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tдом\tдом\tnoun:nom:sg\n\
         F\tдом\tдома\tnoun:gen:sg\n\
         F\tдома\tдома\tnoun:nom:pl\n\
         F\tдома\tдомой\tnoun:dat:pl\n\
         T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
         T\tдом\tдома\tnoun:gen:sg\tнет _\n\
         T\tдома\tдома\tnoun:nom:pl\tработаю _\n\
         H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("дом", 31), ("дома", 7), ("домой", 19)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");

    let terminal_form = |terminal_id| {
        field
            .form_by_terminal
            .binary_search_by_key(&terminal_id, |(terminal_id, _)| *terminal_id)
            .ok()
            .map(|index| field.form_by_terminal[index].1)
    };
    assert_eq!(
        terminal_form(7)
            .and_then(|form_ref| field.decode_form_ref(form_ref))
            .as_deref(),
        Some("дома")
    );
    assert_eq!(terminal_form(999), None);

    let shared_form = field.form_ref_for_surface("дома").expect("shared form");
    let shared_lemmas = field
        .bindings_for_form(shared_form)
        .map(|binding| binding.lemma_center_id)
        .collect::<Vec<_>>();
    assert_eq!(shared_lemmas.len(), 2);
    assert_ne!(shared_lemmas[0], shared_lemmas[1]);

    for lemma_id in shared_lemmas {
        let bindings = field.bindings_for_lemma(lemma_id).collect::<Vec<_>>();
        assert!(!bindings.is_empty());
        assert!(bindings
            .iter()
            .all(|binding| binding.lemma_center_id == lemma_id));
        assert!(bindings
            .iter()
            .any(|binding| binding.form_center_ref == shared_form));
    }
}

#[test]
fn lexical_lemma_observation_exposes_complete_typed_read_only_sources() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tlemma-a\tform-a\tnoun:nom:sg\n\
         F\tlemma-a\tform-b\tnoun:gen:sg\n\
         F\tlemma-a\tform-c\tverb:inf\n\
         T\tlemma-a\tform-a\tnoun:nom:sg\t_ context\n\
         H\tlemma-a\tform-b\tnoun:gen:sg\theldout _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("form-a", 1), ("form-b", 2), ("form-c", 3)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");

    let observation = field
        .lexical_lemma_observation_v1(0)
        .expect("observation")
        .expect("known lemma");

    assert_eq!(observation.lemma_id, 0);
    assert_eq!(observation.known_pos_domains, vec![1, 2]);
    assert_eq!(observation.exact_source_forms.len(), 3);
    assert_eq!(
        observation.canonical_source_form_ref,
        observation
            .exact_source_forms
            .first()
            .map(|source| source.form_ref)
    );
    assert!(observation.exact_source_forms.windows(2).all(|pair| {
        (
            pair[0].canonical_preference,
            pair[0].normalized_surface.chars().count(),
            pair[0].feature_mask,
            &pair[0].normalized_surface,
            pair[0].form_ref,
        ) <= (
            pair[1].canonical_preference,
            pair[1].normalized_surface.chars().count(),
            pair[1].feature_mask,
            &pair[1].normalized_surface,
            pair[1].form_ref,
        )
    }));
    assert_eq!(
        field
            .lexical_lemma_observation_v1(1)
            .expect("unknown lemma"),
        None
    );
}

#[test]
fn compositional_birth_recovers_an_unbound_exact_paradigm_surface() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tпроверять\tпроверять\tverb:inf:imperf\n\
         F\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\n\
         F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
         T\tпроверять\tпроверять\tverb:inf:imperf\tнужно _\n\
         H\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\tя _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("проверять", 17)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");

    let births = field.compositional_form_births("провряю", 4, 8);
    for limit in 1..=births.len() {
        assert_eq!(
            field.compositional_form_births("провряю", 4, limit),
            births.iter().copied().take(limit).collect::<Vec<_>>(),
            "deferred atom scoring must preserve eager full-lattice order"
        );
    }
    let target = field
        .form_ref_for_surface("проверяю")
        .expect("exact target surface");
    assert!(births.iter().any(|birth| birth.form_ref == target));
    assert_eq!(field.l1_terminal_for_form_ref(target), None);

    let seeds = births
        .iter()
        .map(|birth| L2LexicalSeed {
            terminal_id: None,
            surface: field
                .decode_form_ref(birth.form_ref)
                .map(|value| value.into_owned()),
            evidence_milli: i32::from(birth.evidence_milli),
            origin: L2LexicalSeedOrigin::CompositionalMorphology,
        })
        .collect::<Vec<_>>();
    let readout = field.readout("я _", &seeds, 8);
    assert_eq!(readout.verdict, L2LocalVerdict::Abstain);
    assert!(readout
        .candidates
        .iter()
        .any(|candidate| candidate.form_ref == target));

    let clean = field.compositional_form_births("ПРОВЕРЯЕТ!", 1, 1);
    let clean_target = field
        .form_ref_for_surface("проверяет")
        .expect("normalized clean target");
    assert_eq!(
        clean.first().map(|birth| birth.form_ref),
        Some(clean_target)
    );
    assert_eq!(clean[0].evidence_milli, 1_000);
    assert_eq!(clean[0].wave_distance, 0);
}

#[test]
fn contextual_lemma_reduction_uses_trained_slot_evidence_before_form_expansion() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tпроверка\tпроверка\tnoun:nom:sg\n\
         F\tпроверка\tпроверки\tnoun:gen:sg\n\
         F\tпроверять\tпроверять\tverb:inf:imperf\n\
         F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
         T\tпроверка\tпроверки\tnoun:gen:sg\tнет _\n\
         T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n\
         H\tпроверка\tпроверка\tnoun:nom:sg\t_ готова\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([
        ("проверка", 11),
        ("проверки", 13),
        ("проверять", 17),
        ("проверяет", 19),
    ]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let noun_lemma = field
        .form_ref_for_surface("проверка")
        .and_then(|form_ref| field.bindings_for_form(form_ref).next())
        .map(|binding| binding.lemma_center_id)
        .expect("noun lemma");
    let verb_lemma = field
        .form_ref_for_surface("проверяет")
        .and_then(|form_ref| field.bindings_for_form(form_ref).next())
        .map(|binding| binding.lemma_center_id)
        .expect("verb lemma");
    let broad = vec![
        CompositionalLemmaBirth {
            lemma_id: noun_lemma,
            atom_evidence: 100,
            atom_evidence_milli: 1_000,
            wave_distance: 1,
        },
        CompositionalLemmaBirth {
            lemma_id: verb_lemma,
            atom_evidence: 90,
            atom_evidence_milli: 900,
            wave_distance: 2,
        },
    ];

    let active = field.contextual_compositional_lemma_births("он _", &broad, 1);
    assert_eq!(active.first().map(|birth| birth.lemma_id), Some(verb_lemma));
    assert_eq!(
        field.contextual_compositional_lemma_births("он _", &broad, broad.len()),
        broad,
        "a fully retained lemma lattice must not pay for or change a 256 -> 256 rerank"
    );

    let unmatched = field.contextual_compositional_lemma_births("совсем другой _", &broad, 1);
    assert_eq!(
        unmatched.first().map(|birth| birth.lemma_id),
        Some(noun_lemma)
    );
}

#[test]
fn contextual_form_expansion_selects_trained_slots_without_dropping_lemma_basins() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tпроверять\tпроверять\tverb:inf:imperf\n\
         F\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\n\
         F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
         T\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\tя _\n\
         T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n\
         H\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("проверять", 17), ("проверяю", 19), ("проверяет", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let first_person = field
        .form_ref_for_surface("проверяю")
        .expect("first-person form");
    let third_person = field
        .form_ref_for_surface("проверяет")
        .expect("third-person form");
    let lemma_id = field
        .bindings_for_form(third_person)
        .next()
        .map(|binding| binding.lemma_center_id)
        .expect("verb lemma");
    let broad = [CompositionalLemmaBirth {
        lemma_id,
        atom_evidence: 100,
        atom_evidence_milli: 1_000,
        wave_distance: 1,
    }];

    let one_slot =
        field.contextual_compositional_form_births_from_lemmas("он _", "провераю", &broad, 1, 8);
    assert!(one_slot.iter().any(|birth| birth.form_ref == third_person));
    assert!(!one_slot.iter().any(|birth| birth.form_ref == first_person));

    let two_slots =
        field.contextual_compositional_form_births_from_lemmas("он _", "провераю", &broad, 2, 8);
    assert!(two_slots.iter().any(|birth| birth.form_ref == third_person));
    assert!(two_slots.iter().any(|birth| birth.form_ref == first_person));
}

#[test]
fn uncontextualized_feature_selection_uses_surface_geometry_before_support() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\n\
         F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
         T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n\
         T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tпроцесс _\n\
         H\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\tя _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("проверяю", 19), ("проверяет", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let first_person = field
        .form_ref_for_surface("проверяю")
        .expect("first-person form");
    let lemma_id = field
        .bindings_for_form(first_person)
        .next()
        .map(|binding| binding.lemma_center_id)
        .expect("verb lemma");
    let broad = [CompositionalLemmaBirth {
        lemma_id,
        atom_evidence: 100,
        atom_evidence_milli: 1_000,
        wave_distance: 1,
    }];
    let unknown_context = "неизвестная сцена _";
    assert!(!field.context_mode_known(unknown_context));

    let births = field.contextual_compositional_form_births_from_lemmas(
        unknown_context,
        "провераю",
        &broad,
        1,
        8,
    );

    assert_eq!(
        births.first().map(|birth| birth.form_ref),
        Some(first_person)
    );
    assert!(births.iter().all(|birth| birth.form_ref == first_person));
}

#[test]
fn inverse_single_edit_lane_finds_package_forms_without_scanning_the_field() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tокно\tокно\tnoun:nom:sg\n\
         F\tокно\tокне\tnoun:prep:sg\n\
         F\tперспективный\tперспективнее\tadj:comp\n\
         F\tотвлекаться\tотвлекайся\tverb:imp:p2:sg:imperf\n\
         F\tперехватить\tперехвачу\tverb:fut:ind:p1:sg:perf\n\
         T\tокно\tокно\tnoun:nom:sg\t_ открыто\n\
         T\tокно\tокне\tnoun:prep:sg\tв _\n\
         H\tокно\tокне\tnoun:prep:sg\tна _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([
        ("окно", 7),
        ("окне", 11),
        ("перехвачу", 13),
        ("перспективнее", 17),
        ("отвлекайся", 19),
    ]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");

    let surfaces = |damaged| {
        field
            .single_edit_form_refs(damaged, 16)
            .into_iter()
            .filter_map(|form_ref| field.decode_form_ref(form_ref))
            .collect::<Vec<_>>()
    };
    assert_eq!(surfaces("окное"), vec!["окне", "окно"]);
    assert_eq!(surfaces("перхвачу"), vec!["перехвачу"]);
    assert_eq!(surfaces("переспективнее"), vec!["перспективнее"]);
    assert_eq!(surfaces("отвликайся"), vec!["отвлекайся"]);
    assert!(surfaces("окне").is_empty(), "clean forms must not fan out");
}

#[test]
fn inverse_geometry_birth_is_attenuated_and_cannot_exist_without_grounded_l11() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tкод\tкод\tnoun:nom:sg\n\
         F\tкот\tкот\tnoun:nom:sg\n\
         T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
         T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
         H\tкод\tкод\tnoun:nom:sg\tпроверяю _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("код", 17), ("кот", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let grounded = L2LexicalSeed {
        terminal_id: Some(17),
        surface: None,
        evidence_milli: 1_000,
        origin: L2LexicalSeedOrigin::GroundedL11,
    };
    let inverse = L2LexicalSeed {
        terminal_id: Some(23),
        surface: None,
        evidence_milli: 1_000,
        origin: L2LexicalSeedOrigin::InverseGeometry,
    };

    let readout = field.readout("неизвестная сцена _", &[grounded, inverse.clone()], 8);
    let grounded_candidate = readout
        .candidates
        .iter()
        .find(|candidate| candidate.surface == "код")
        .expect("grounded candidate");
    let inverse_candidate = readout
        .candidates
        .iter()
        .find(|candidate| candidate.surface == "кот")
        .expect("inverse candidate");
    assert_eq!(grounded_candidate.l1_evidence_milli, 1_000);
    assert_eq!(inverse_candidate.l1_evidence_milli, 760);

    let inverse_only = field.readout("_ спит", &[inverse], 8);
    assert_eq!(inverse_only.verdict, L2LocalVerdict::Abstain);
    assert!(inverse_only.candidates.is_empty());
}

#[test]
fn observed_readout_promotes_only_a_unique_exact_compositional_basin() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tсигнал\tсигнал\tnoun:nom:sg\n\
         F\tсигнал\tсигналы\tnoun:nom:pl\n\
         T\tсигнал\tсигнал\tnoun:nom:sg\t_ принят\n\
         H\tсигнал\tсигнал\tnoun:nom:sg\t_ получен\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("сигналы", 17)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let grounded = L2LexicalSeed {
        terminal_id: Some(17),
        surface: None,
        evidence_milli: 1_000,
        origin: L2LexicalSeedOrigin::GroundedL11,
    };
    let compositional = L2LexicalSeed {
        terminal_id: None,
        surface: Some("сигнал".to_string()),
        evidence_milli: 900,
        origin: L2LexicalSeedOrigin::CompositionalMorphology,
    };

    let readout = field.readout_observed(
        "неизвестная сцена _",
        "сигна",
        &[grounded, compositional.clone()],
        8,
    );
    let target = field.form_ref_for_surface("сигнал").expect("target form");
    assert_eq!(readout.verdict, L2LocalVerdict::Winner { form_ref: target });

    let composition_only =
        field.readout_observed("неизвестная сцена _", "сигна", &[compositional], 8);
    assert_eq!(composition_only.verdict, L2LocalVerdict::Abstain);
}

#[test]
fn observed_readout_keeps_multiple_exact_basins_tied() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tкод\tкод\tnoun:nom:sg\n\
         F\tкод\tкоды\tnoun:nom:pl\n\
         F\tкот\tкот\tnoun:nom:sg\n\
         T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
         T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
         H\tкод\tкод\tnoun:nom:sg\tпроверен _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("коды", 17), ("кот", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let seeds = [
        L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 1_000,
            origin: L2LexicalSeedOrigin::GroundedL11,
        },
        L2LexicalSeed {
            terminal_id: None,
            surface: Some("код".to_string()),
            evidence_milli: 900,
            origin: L2LexicalSeedOrigin::CompositionalMorphology,
        },
        L2LexicalSeed {
            terminal_id: None,
            surface: Some("кот".to_string()),
            evidence_milli: 900,
            origin: L2LexicalSeedOrigin::CompositionalMorphology,
        },
    ];

    let readout = field.readout_observed("неизвестная сцена _", "кок", &seeds, 8);
    let code = field.form_ref_for_surface("код").expect("code form");
    let cat = field.form_ref_for_surface("кот").expect("cat form");
    let tied = match readout.verdict {
        L2LocalVerdict::Tied { form_refs } => form_refs,
        verdict => panic!(
            "expected exact geometry tie for {code} and {cat}, got {verdict:?}; candidates={:?}",
            readout.candidates
        ),
    };
    assert_eq!(tied.len(), 2);
    assert!(tied.contains(&code));
    assert!(tied.contains(&cat));
}

#[test]
fn standalone_field_walks_from_l1_seed_to_contextual_same_lemma_form() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tдом\tдом\tnoun:nom:sg\n\
         F\tдом\tдома\tnoun:gen:sg\n\
         T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
         T\tдом\tдома\tnoun:gen:sg\tнет _\n\
         H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let bytes = encode_package(&package).expect("encode");
    let field = StandaloneL2Field::from_bytes(&bytes).expect("load");
    let readout = field.readout(
        "нет _",
        &[L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 900,
            origin: L2LexicalSeedOrigin::GroundedL11,
        }],
        8,
    );

    assert_eq!(field.l1_package_fingerprint(), 99);
    assert_eq!(readout.verdict, L2LocalVerdict::Winner { form_ref: 1 });
    assert_eq!(readout.candidates[0].surface, "дома");
}

#[test]
fn standalone_field_materializes_a_form_that_is_absent_from_l1() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tдом\tдом\tnoun:nom:sg\n\
         F\tдом\tдома\tnoun:gen:sg\n\
         T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
         T\tдом\tдома\tnoun:gen:sg\tнет _\n\
         H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
    )
    .expect("teacher");
    let (package, report) =
        compile_l2_package(&corpus, 99, |surface| (surface == "дом").then_some(17))
            .expect("compile");
    assert_eq!(report.l1_bound_forms, 1);
    assert_eq!(report.admitted_forms, 2);
    let field = StandaloneL2Field::from_package(package).expect("load");
    let readout = field.readout(
        "нет _",
        &[L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 900,
            origin: L2LexicalSeedOrigin::GroundedL11,
        }],
        8,
    );

    let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
        panic!("context should settle the generated form");
    };
    let winner = readout
        .candidates
        .iter()
        .find(|candidate| candidate.form_ref == form_ref)
        .expect("winner candidate");
    assert_eq!(winner.surface, "дома");
    assert_eq!(winner.l1_terminal_id, None);
}

#[test]
fn standalone_field_resolves_append_only_l1_seed_by_surface() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tрефакторинг\tрефакторинг\tnoun:nom:sg\n\
         F\tрефакторинг\tрефакторинга\tnoun:gen:sg\n\
         T\tрефакторинг\tрефакторинг\tnoun:nom:sg\t_ нужен\n\
         H\tрефакторинг\tрефакторинга\tnoun:gen:sg\tпроект _\n",
    )
    .expect("teacher");
    let (package, _) = compile_l2_package(&corpus, 99, |surface| {
        (surface == "рефакторинг").then_some(17)
    })
    .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let readout = field.readout(
        "проект _",
        &[L2LexicalSeed {
            terminal_id: Some(900_000),
            surface: Some("рефакторинга".to_string()),
            evidence_milli: 1_000,
            origin: L2LexicalSeedOrigin::GroundedL11,
        }],
        8,
    );

    assert!(readout
        .candidates
        .iter()
        .any(|candidate| candidate.surface == "рефакторинга"));
}

#[test]
fn learned_competition_overcomes_one_reconstruction_attenuation() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tпосмотреть\tпосмотреть\tverb:inf:perf\n\
         F\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\n\
         F\tпросмотреть\tпросмотреть\tverb:inf:perf\n\
         F\tпросмотреть\tпросмотри\tverb:imp_excl:sg:imp:perf\n\
         T\tпосмотреть\tпосмотреть\tverb:inf:perf\tхочу _\n\
         H\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\n\
         NT\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n\
         NH\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("посмотреть", 17), ("просмотреть", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let readout = field.readout(
        "_ сюда",
        &[L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 1_000,
            origin: L2LexicalSeedOrigin::GroundedL11,
        }],
        8,
    );

    let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
        panic!("learned competition should settle one reconstruction: {readout:#?}");
    };
    assert_eq!(field.decode_form_ref(form_ref).as_deref(), Some("посмотри"));
}

#[test]
fn standalone_field_abstains_when_context_mode_is_unknown() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tдом\tдом\tnoun:nom:sg\n\
         F\tдом\tдома\tnoun:gen:sg\n\
         T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
         H\tдом\tдома\tnoun:gen:sg\tнет _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let readout = field.readout(
        "совсем неизвестная сцена _",
        &[L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 900,
            origin: L2LexicalSeedOrigin::GroundedL11,
        }],
        8,
    );
    assert_eq!(readout.verdict, L2LocalVerdict::Abstain);
}

#[test]
fn unknown_context_keeps_multiple_direct_surface_seeds_tied() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tкод\tкод\tnoun:nom:sg\n\
         F\tкот\tкот\tnoun:nom:sg\n\
         T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
         T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
         H\tкод\tкод\tnoun:nom:sg\tпроверяю _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("код", 17), ("кот", 23)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let readout = field.readout(
        "совсем неизвестная сцена _",
        &[
            L2LexicalSeed {
                terminal_id: None,
                surface: Some("код".to_string()),
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            },
            L2LexicalSeed {
                terminal_id: None,
                surface: Some("кот".to_string()),
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            },
        ],
        8,
    );

    assert_eq!(
        readout.verdict,
        L2LocalVerdict::Tied {
            form_refs: vec![0, 1]
        }
    );
}

#[test]
fn contextual_multi_lemma_birth_can_select_a_weaker_seeded_lemma() {
    let corpus = L2TeacherCorpus::parse_tsv(
        "F\tдом\tдом\tnoun:nom:sg\n\
         F\tдом\tдома\tnoun:gen:sg\n\
         F\tдома\tдома\tnoun:nom:sg\n\
         F\tдома\tдомик\tnoun:acc:sg\n\
         T\tдома\tдомик\tnoun:acc:sg\tвижу _\n\
         NT\tдома\tдомик\tnoun:acc:sg\tвижу _\tдома\n\
         H\tдом\tдома\tnoun:gen:sg\tнет _\n",
    )
    .expect("teacher");
    let terminals = BTreeMap::from([("дом", 17), ("дома", 23), ("домик", 31)]);
    let (package, _) = compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
        .expect("compile");
    let field = StandaloneL2Field::from_package(package).expect("load");
    let readout = field.readout(
        "вижу _",
        &[
            L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            },
            L2LexicalSeed {
                terminal_id: Some(23),
                surface: None,
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            },
        ],
        8,
    );

    assert!(
        readout
            .candidates
            .iter()
            .any(|candidate| candidate.surface == "домик"),
        "{readout:#?}"
    );
    let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
        panic!("contextual slot should settle the weaker seeded lemma");
    };
    assert_eq!(
        readout
            .candidates
            .iter()
            .find(|candidate| candidate.form_ref == form_ref)
            .map(|candidate| candidate.surface.as_str()),
        Some("домик")
    );
}

#[test]
fn equal_slot_evidence_within_one_lemma_cannot_become_false_singleton() {
    let candidates = vec![
        L2LocalCandidate {
            form_ref: 17,
            l1_terminal_id: Some(17),
            surface: "первый".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 1_128,
            neighbor_pressure: 0,
            competition_pressure: 128,
            explicit_competition_pressure: 0,
            local_score: 2_256,
            lemma_ids: vec![3, 7],
            feature_masks: vec![11],
        },
        L2LocalCandidate {
            form_ref: 23,
            l1_terminal_id: Some(23),
            surface: "второй".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 1_128,
            neighbor_pressure: 0,
            competition_pressure: 0,
            explicit_competition_pressure: 0,
            local_score: 2_128,
            lemma_ids: vec![7],
            feature_masks: vec![11],
        },
    ];

    assert_eq!(
        classify_local(
            &candidates,
            TieCalibration {
                minimum_positive: 1,
                minimum_margin: 1,
                tie_window: 1,
                ..TieCalibration::default()
            },
        ),
        L2LocalVerdict::Tied {
            form_refs: vec![17, 23]
        }
    );
}

#[test]
fn competition_alone_cannot_create_cross_lemma_authority() {
    let candidates = vec![
        L2LocalCandidate {
            form_ref: 17,
            l1_terminal_id: Some(17),
            surface: "чужая".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 1_000,
            neighbor_pressure: 0,
            competition_pressure: 500,
            explicit_competition_pressure: 0,
            local_score: 2_500,
            lemma_ids: vec![2],
            feature_masks: vec![11],
        },
        L2LocalCandidate {
            form_ref: 23,
            l1_terminal_id: Some(23),
            surface: "целевая".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 1_000,
            neighbor_pressure: 0,
            competition_pressure: 0,
            explicit_competition_pressure: 0,
            local_score: 2_000,
            lemma_ids: vec![1],
            feature_masks: vec![11],
        },
    ];

    assert_eq!(
        classify_local(
            &candidates,
            TieCalibration {
                minimum_positive: 1,
                minimum_margin: 1,
                tie_window: 1,
                ..TieCalibration::default()
            },
        ),
        L2LocalVerdict::Tied {
            form_refs: vec![17, 23]
        }
    );
}

#[test]
fn explicit_competition_without_a_winner_lemma_seed_stays_tied() {
    let candidates = vec![
        L2LocalCandidate {
            form_ref: 17,
            l1_terminal_id: None,
            surface: "чужая".to_string(),
            l1_evidence_milli: 760,
            slot_phase_milli: 1_000,
            neighbor_pressure: 0,
            competition_pressure: 500,
            explicit_competition_pressure: 500,
            local_score: 2_260,
            lemma_ids: vec![2],
            feature_masks: vec![11],
        },
        L2LocalCandidate {
            form_ref: 23,
            l1_terminal_id: Some(23),
            surface: "целевая".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 1_000,
            neighbor_pressure: 0,
            competition_pressure: 0,
            explicit_competition_pressure: 0,
            local_score: 2_000,
            lemma_ids: vec![1],
            feature_masks: vec![11],
        },
    ];

    assert_eq!(
        classify_local(
            &candidates,
            TieCalibration {
                minimum_positive: 1,
                minimum_margin: 1,
                tie_window: 1,
                ..TieCalibration::default()
            },
        ),
        L2LocalVerdict::Tied {
            form_refs: vec![17, 23]
        }
    );
}

#[test]
fn inclusive_imperative_variants_in_one_lemma_are_tied() {
    let inclusive_singular =
        crate::nanda_wave::morphology_phase::parse_features("verb:imp_incl:sg:imp:perf")
            .expect("inclusive singular");
    let inclusive_plural =
        crate::nanda_wave::morphology_phase::parse_features("verb:imp_incl:pl:imp:perf")
            .expect("inclusive plural");
    let candidates = vec![
        L2LocalCandidate {
            form_ref: 17,
            l1_terminal_id: Some(17),
            surface: "первый".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 1_088,
            neighbor_pressure: 0,
            competition_pressure: 0,
            explicit_competition_pressure: 0,
            local_score: 2_088,
            lemma_ids: vec![7],
            feature_masks: vec![inclusive_singular],
        },
        L2LocalCandidate {
            form_ref: 23,
            l1_terminal_id: Some(23),
            surface: "второй".to_string(),
            l1_evidence_milli: 1_000,
            slot_phase_milli: 0,
            neighbor_pressure: 0,
            competition_pressure: 0,
            explicit_competition_pressure: 0,
            local_score: 1_000,
            lemma_ids: vec![7],
            feature_masks: vec![inclusive_plural],
        },
    ];

    assert_eq!(
        classify_local(
            &candidates,
            TieCalibration {
                minimum_positive: 1,
                minimum_margin: 1,
                tie_window: 1,
                ..TieCalibration::default()
            },
        ),
        L2LocalVerdict::Tied {
            form_refs: vec![17, 23]
        }
    );
}
