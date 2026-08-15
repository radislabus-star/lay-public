use super::super::super::restoration::{RestorationCalibration, RestorationCandidate};
use super::super::super::runtime::RECONSTRUCTION_MODE_DELETION_TRANSPOSITION;
use super::super::super::{compiler, format, v8, v9};
use super::*;

fn candidate(
    terminal_id: u32,
    geometry_distance: u8,
    reconstruction_modes: u8,
) -> GrokkingCandidate {
    GrokkingCandidate {
        terminal_id,
        geometry_distance,
        reconstruction_modes,
        positive_milli: 1_000,
        backward_milli: 1_000,
        ..GrokkingCandidate::default()
    }
}

fn output(candidates: Vec<GrokkingCandidate>) -> SurfaceEvaluation {
    let typed_certificate_classes = candidates
        .iter()
        .map(|candidate| (candidate.terminal_id, vec!["fixture"]))
        .collect();
    let implicit_activations = candidates
        .iter()
        .map(|candidate| {
            (
                candidate.terminal_id,
                ForwardActivation {
                    mass: u64::from(candidate.terminal_id),
                    hits: 1,
                    surface_hits: 1,
                    keyboard_hits: 0,
                },
            )
        })
        .collect();
    let readout = restoration::classify(&candidates, RestorationCalibration::LEGACY_PERMISSIVE);
    SurfaceEvaluation {
        exact: ExactSettlementResult {
            candidates,
            readout,
            phase_noop: true,
            reverse_terminals: 1,
            reverse_relations: 1,
            elapsed_us: 1,
        },
        typed_certificate_classes,
        implicit_activations,
        typed_us: 1,
        implicit_us: 1,
        legacy_candidates: Vec::new(),
        legacy_readout: None,
        legacy_us: 0,
    }
}

fn fixed_case(terminal_id: u32) -> FixedHeldoutCase {
    FixedHeldoutCase {
        class: "missing_letter",
        terminal_id,
        surface: "fixture".to_string(),
    }
}

#[test]
fn gate_c_bounded_projection_rescues_certificate_below_raw_projection_rank() {
    let mut candidates = (0..80)
        .map(|terminal| candidate(terminal, 0, 0))
        .collect::<Vec<_>>();
    candidates[70].reconstruction_modes = RECONSTRUCTION_MODE_DELETION_TRANSPOSITION;
    let output = output(candidates);
    let before = output.exact.candidates.clone();
    let mut shard = QualityShard::default();
    let words = (0..80).map(|value| value.to_string()).collect::<Vec<_>>();
    record_damaged(
        &mut shard,
        &words,
        &fixed_case(70),
        &BTreeSet::from([70]),
        &output,
    );
    let metrics = shard.classes["missing_letter"].clone();
    assert_eq!(metrics.target_retained, 1);
    assert_eq!(metrics.target_in_projection, 1);
    assert_eq!(output.exact.candidates, before);
}

#[test]
fn gate_c_bounded_projection_does_not_rescue_ungrounded_tail() {
    let candidates = (0..80)
        .map(|terminal| candidate(terminal, 0, 0))
        .collect::<Vec<_>>();
    let output = output(candidates);
    let before = output.exact.candidates.clone();
    let mut shard = QualityShard::default();
    let words = (0..80).map(|value| value.to_string()).collect::<Vec<_>>();

    record_damaged(
        &mut shard,
        &words,
        &fixed_case(70),
        &BTreeSet::from([70]),
        &output,
    );

    assert_eq!(shard.classes["missing_letter"].target_in_projection, 0);
    assert_eq!(output.exact.candidates, before);
}

#[test]
fn gate_c_objective_is_read_only_after_readout() {
    let output = output(vec![candidate(1, 0, 0), candidate(2, 1, 0)]);
    let exact_before = output.exact.candidates.clone();
    let readout_before = output.exact.readout.clone();
    let words = vec!["zero".to_string(), "one".to_string(), "two".to_string()];
    let mut first = QualityShard::default();
    let mut second = QualityShard::default();
    record_damaged(
        &mut first,
        &words,
        &fixed_case(1),
        &BTreeSet::from([1]),
        &output,
    );
    record_damaged(
        &mut second,
        &words,
        &fixed_case(1),
        &BTreeSet::from([1, 2]),
        &output,
    );
    assert_eq!(output.exact.candidates, exact_before);
    assert_eq!(output.exact.readout, readout_before);
    assert_ne!(first.classes, second.classes);
}

#[test]
fn gate_c_legacy_observer_cannot_change_exact_metrics() {
    let without_legacy = output(vec![candidate(1, 0, 0)]);
    let mut with_legacy = output(vec![candidate(1, 0, 0)]);
    let foreign = candidate(2, 0, 1);
    with_legacy.legacy_candidates.push(foreign);
    with_legacy.legacy_readout = Some(RestorationReadout::Winner {
        candidate: RestorationCandidate::from(&foreign),
    });
    let words = vec!["zero".to_string(), "one".to_string(), "two".to_string()];
    let case = fixed_case(1);
    let objective = BTreeSet::from([1]);
    let mut first = QualityShard::default();
    let mut second = QualityShard::default();
    record_damaged(&mut first, &words, &case, &objective, &without_legacy);
    record_damaged(&mut second, &words, &case, &objective, &with_legacy);
    let first = &first.classes["missing_letter"];
    let second = &second.classes["missing_letter"];
    assert_eq!(first.target_retained, second.target_retained);
    assert_eq!(first.unique_top1, second.unique_top1);
    assert_eq!(first.false_authority, second.false_authority);
    assert_eq!(second.legacy_grounded_losses, 1);
}

#[test]
fn gate_c_false_authority_and_false_singleton_are_independent() {
    let first = candidate(1, 0, 0);
    let second = candidate(2, 0, 0);
    let mut output = output(vec![first, second]);
    output.exact.readout = RestorationReadout::Winner {
        candidate: RestorationCandidate::from(&second),
    };
    let words = vec!["zero".to_string(), "one".to_string(), "two".to_string()];
    let mut shard = QualityShard::default();
    record_damaged(
        &mut shard,
        &words,
        &fixed_case(1),
        &BTreeSet::from([1]),
        &output,
    );
    let metrics = &shard.classes["missing_letter"];
    assert_eq!(metrics.false_authority, 1);
    assert_eq!(metrics.false_singleton, 1);
}

#[test]
fn gate_c_loss_diagnostics_expose_bounded_numeric_evidence() {
    let mut target = candidate(1, 2, 1);
    target.forward_milli = 901;
    target.backward_milli = 902;
    target.structural_milli = 903;
    target.sequence_milli = 904;
    target.position_milli = 905;
    target.length_milli = 906;
    target.settled_energy = 907;
    let output = output(vec![candidate(2, 1, 0), target]);
    let before = output.exact.candidates.clone();
    let words = vec!["zero".to_string(), "target".to_string(), "top".to_string()];
    let mut shard = QualityShard::default();

    record_damaged(
        &mut shard,
        &words,
        &fixed_case(1),
        &BTreeSet::from([1]),
        &output,
    );

    let diagnostic = shard
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.mechanism == "unique_objective_not_rank1")
        .unwrap();
    assert_eq!(diagnostic.top_candidate_evidence.len(), 2);
    assert_eq!(diagnostic.top_candidate_evidence[0].surface, "top");
    assert_eq!(
        diagnostic.top_candidate_evidence[1].typed_certificate_classes,
        vec!["fixture"]
    );
    assert_eq!(
        diagnostic.top_candidate_evidence[1]
            .implicit_activation
            .unwrap()
            .mass,
        1
    );
    assert_eq!(diagnostic.top_candidate_evidence[1].settled_energy, 907);
    assert_eq!(output.exact.candidates, before);
}

#[test]
fn gate_c_parallel_merge_matches_single_shard() {
    let left = ClassQuality {
        cases: 1,
        target_retained: 1,
        ..ClassQuality::default()
    };
    let right = ClassQuality {
        cases: 2,
        target_retained: 2,
        ..ClassQuality::default()
    };
    let mut expected = left.clone();
    expected.merge(right.clone());
    let mut first = QualityShard::default();
    first.classes.insert("missing_letter", left);
    let mut second = QualityShard::default();
    second.classes.insert("missing_letter", right);
    first.merge(second);
    assert_eq!(first.classes["missing_letter"], expected);
}

#[test]
fn gate_c_thresholds_keep_strict_and_non_strict_semantics() {
    assert!(!ratio_strictly_above(95, 100, 95, 100));
    assert!(ratio_strictly_above(96, 100, 95, 100));
    assert!(ratio_at_least(99, 100, 99, 100));
    assert!(ratio_at_least(999, 1_000, 999, 1_000));
}

#[test]
fn gate_c_clean_smoke_sampling_is_deterministic_and_bounded() {
    assert_eq!(clean_terminal_ids(10, 4), vec![0, 2, 5, 7]);
    assert_eq!(clean_terminal_ids(4, 0), vec![0, 1, 2, 3]);
    assert_eq!(clean_terminal_ids(4, 10), vec![0, 1, 2, 3]);
}

#[test]
fn gate_c_cli_is_separate_from_a2_command() {
    let cli = include_str!("../../../../bin/lay_nanda_wave_train.rs");
    assert!(cli.contains("--prove-l1-typed-basin-implicit-forward"));
    assert!(cli.contains("--prove-l1-typed-basin-quality"));
    assert!(cli.contains("--damage-class"));
}

#[test]
fn gate_c_class_filter_is_scheduler_only_and_cannot_claim_full_pass() {
    let cases = vec![
        fixed_case(1),
        FixedHeldoutCase {
            class: "extra_letter",
            terminal_id: 2,
            surface: "fixture-two".to_string(),
        },
    ];

    let selected = select_damage_cases(cases, Some("extra_letter")).unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].class, "extra_letter");
    assert_eq!(
        quality_verdict(true, false, true, false),
        "DIAGNOSTIC_C_CLASS"
    );
    assert_ne!(quality_verdict(true, false, true, false), "PASS_C_QUALITY");
}

#[test]
fn gate_c_direct_v9_uses_artifact_support_only_after_corpus_parity() {
    let (root, v9_path, words) = direct_v9_fixture("support-parity");
    let layout = read_quality_artifact_layout(&v9_path).expect("read direct V9 layout");
    let memory = LexicalGrokkingMemory::load(&v9_path).expect("load direct V9 fixture");
    let rebuilt = ExactSupportField::rebuild(&memory.package, &words)
        .expect("rebuild exact support from fixture corpus");
    let projected = layout
        .base_bytes
        .saturating_add(rebuilt.metrics.projected_overflow_bytes as u64);

    assert_eq!(layout.format, QualityArtifactFormat::V9);
    assert!(direct_v9_support_matches_rebuild(
        layout,
        memory.typed_basin_support(),
        &rebuilt,
        projected,
    ));

    let mut wrong_words = words;
    wrong_words[0].push('x');
    let mismatched = ExactSupportField::rebuild(&memory.package, &wrong_words)
        .expect("rebuild mismatched fixture support");
    assert!(!direct_v9_support_matches_rebuild(
        layout,
        memory.typed_basin_support(),
        &mismatched,
        layout
            .base_bytes
            .saturating_add(mismatched.metrics.projected_overflow_bytes as u64),
    ));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn gate_c_v8_and_v9_layout_routes_are_distinct() {
    let (root, v9_path, _) = direct_v9_fixture("format-route");
    let v8_path = root.join("fixture.v8.bin");

    assert_eq!(
        read_quality_artifact_layout(&v8_path)
            .expect("read V8 layout")
            .format,
        QualityArtifactFormat::V8
    );
    assert_eq!(
        read_quality_artifact_layout(&v9_path)
            .expect("read V9 layout")
            .format,
        QualityArtifactFormat::V9
    );

    let _ = std::fs::remove_dir_all(root);
}

fn direct_v9_fixture(label: &str) -> (std::path::PathBuf, std::path::PathBuf, Vec<String>) {
    let words = vec![
        "время".to_string(),
        "работает".to_string(),
        "download".to_string(),
    ];
    let training = words
        .iter()
        .enumerate()
        .map(|(terminal_id, surface)| compiler::TrainingWord {
            terminal_id: terminal_id as u32,
            surface: surface.clone(),
            training_surfaces: Vec::new(),
        })
        .collect::<Vec<_>>();
    let package =
        compiler::compile_with_policy(&training, compiler::ForwardPostingPolicy::Complete)
            .expect("compile direct V9 fixture")
            .package;
    let root = std::env::temp_dir().join(format!(
        "lay-quality-direct-v9-{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("create direct V9 fixture directory");
    let v7_path = root.join("fixture.v7.bin");
    let v8_path = root.join("fixture.v8.bin");
    let v9_path = root.join("fixture.v9.bin");
    std::fs::write(
        &v7_path,
        format::encode_compact_depth0(&package).expect("encode direct V9 compact base"),
    )
    .expect("write direct V9 compact base");
    v8::build_lazy_v8_package_with_shard_size(&v7_path, &v8_path, 32)
        .expect("build direct V9 source fixture");
    v9::build_exact_v9_package(&v8_path, &v9_path).expect("build direct V9 fixture");
    (root, v9_path, words)
}
