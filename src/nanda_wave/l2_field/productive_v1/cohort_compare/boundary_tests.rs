use std::sync::{Arc, OnceLock};

use super::*;
use crate::candidate_contract::CandidateOrigin;
use crate::config::CorrectionSafety;
use crate::correction_core::{
    CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, TypingErrorClass,
    TypingErrorEvent, UnifiedCorrectionCandidate,
};
use crate::nanda_wave::l2_field::productive_v1::format::{
    encode_package, ProductiveAlgorithmModeV1, ProductivePackageBuildV1, ProductiveSectionBuildV1,
    ProductiveSectionKindV1, REQUIRED_SECTION_COUNT,
};
use crate::nanda_wave::l2_field::productive_v1::live::{
    CanonicalContourRelation, CanonicalContourSeed,
};
use crate::nanda_wave::l2_field::productive_v1::material_frame::{
    CanonicalL1AnchorProofFaultV1, ExactPeakBirthEnumerationV1, ExactPeakCandidateInputV1,
};
use crate::nanda_wave::l2_field::productive_v1::packaged_runtime::PackagedProductiveRuntimeV1;
use crate::nanda_wave::l2_field::productive_v1::records::{
    DeltaManifestRecordV1, EvidencePriorRecordV1, FixedRecordV1, ModelCoefficientRecordV1,
};
use crate::nanda_wave::l2_field::productive_v1::score::{
    productive_feature_schema_hash_low, PRODUCTIVE_FEATURE_COUNT,
};
use crate::nanda_wave::lexical_grokking::L11SeedSurface;
use crate::typing_transition::decision::{
    DecisionEvidenceMode, TransitionDecisionCore, TransitionDecisionPolicy,
};

const CONTEXT: &str = "нужна ";
const OBSERVED: &str = "форм";
const ORIGINAL: &str = "нужна форм ";
const WINNER: &str = "нужна форма ";

fn replace_fixed<T: FixedRecordV1>(
    build: &mut ProductivePackageBuildV1,
    kind: ProductiveSectionKindV1,
    records: &[T],
) {
    let replacement =
        ProductiveSectionBuildV1::fixed_records(kind, records).expect("valid fixed section");
    *build
        .sections
        .iter_mut()
        .find(|section| section.kind == kind)
        .expect("required productive section") = replacement;
}

fn empty_productive_runtime() -> PackagedProductiveRuntimeV1 {
    let mut build = ProductivePackageBuildV1::with_empty_required_sections(
        ProductiveAlgorithmModeV1::ProductiveV1Model,
    );
    build.l11_package_sha256 = [7; 32];
    build.canonical_l2_package_sha256 = [7; 32];
    build.training_manifest_sha256 = [9; 32];
    build.maximum_observed_scalars = 64;
    build.maximum_generated_scalars = 96;
    build.maximum_program_operations = 1;
    build.normalization_version = 1;
    build.compiler_version = 1;
    build.productive_package_byte_budget = 1 << 20;

    for (kind, magic) in [
        (ProductiveSectionKindV1::AxisDictionaries, b"ADV1"),
        (ProductiveSectionKindV1::SegmentPool, b"SPV1"),
    ] {
        let section = build
            .sections
            .iter_mut()
            .find(|section| section.kind == kind)
            .expect("required pool section");
        section.bytes.extend_from_slice(magic);
        section.bytes.extend_from_slice(&0_u32.to_le_bytes());
    }

    let schema_hash = productive_feature_schema_hash_low().expect("feature schema");
    let coefficients = (1..=PRODUCTIVE_FEATURE_COUNT)
        .map(|feature_id| ModelCoefficientRecordV1 {
            feature_id: feature_id as u16,
            coefficient_q16: 0,
            train_support: 1,
            feature_schema_hash_low: schema_hash,
            ..ModelCoefficientRecordV1::default()
        })
        .collect::<Vec<_>>();
    replace_fixed(
        &mut build,
        ProductiveSectionKindV1::ModelCoefficients,
        &coefficients,
    );
    let priors = (1..=4)
        .map(|channel_id| EvidencePriorRecordV1 {
            channel_id,
            positive_prior_twice: 5,
            contradiction_prior_twice: 1,
            ..EvidencePriorRecordV1::default()
        })
        .collect::<Vec<_>>();
    replace_fixed(&mut build, ProductiveSectionKindV1::EvidencePriors, &priors);
    replace_fixed(
        &mut build,
        ProductiveSectionKindV1::DeltaManifest,
        &[DeltaManifestRecordV1 {
            section_count_ref: REQUIRED_SECTION_COUNT as u64,
            requested_authority_scope: 1,
            ..DeltaManifestRecordV1::default()
        }],
    );

    PackagedProductiveRuntimeV1::from_bytes(
        encode_package(&build).expect("empty productive package"),
        [7; 32],
        [7; 32],
    )
    .expect("empty productive runtime")
}

fn canonical_index() -> crate::nanda_wave::l2_field::runtime::StandaloneL2Field {
    let corpus = crate::nanda_wave::l2_field::teacher::L2TeacherCorpus::parse_tsv(
        "F\tlemma-a\tформа\tnoun:nom:sg\n\
         F\tlemma-a\tформу\tnoun:acc:sg\n\
         F\tlemma-a\tформы\tnoun:gen:sg\n\
         T\tlemma-a\tформа\tnoun:nom:sg\t_ формы\n\
         H\tlemma-a\tформы\tnoun:gen:sg\tформа _\n\
         F\tlemma-cash\tкасса\tnoun:nom:sg\n\
         F\tlemma-left\tобщую\tnoun:acc:sg\n\
         F\tlemma-left\tлевая\tnoun:nom:sg\n\
         F\tlemma-right\tобщую\tnoun:acc:sg\n\
         F\tlemma-right\tправая\tnoun:nom:sg\n",
    )
    .expect("canonical fixture corpus");
    let (package, _) =
        crate::nanda_wave::l2_field::compiler::compile_l2_package(&corpus, 7, |surface| {
            match surface {
                "форма" => Some(17),
                "формы" => Some(18),
                "касса" => Some(19),
                "левая" => Some(20),
                "правая" => Some(21),
                _ => None,
            }
        })
        .expect("canonical fixture package");
    crate::nanda_wave::l2_field::runtime::StandaloneL2Field::from_package(package)
        .expect("canonical fixture index")
}

fn fixture_owners() -> &'static (
    crate::nanda_wave::l2_field::runtime::StandaloneL2Field,
    PackagedProductiveRuntimeV1,
) {
    static OWNERS: OnceLock<(
        crate::nanda_wave::l2_field::runtime::StandaloneL2Field,
        PackagedProductiveRuntimeV1,
    )> = OnceLock::new();
    OWNERS.get_or_init(|| (canonical_index(), empty_productive_runtime()))
}

fn anchor_replay() -> CanonicalL1AnchorReplayContextV1<'static> {
    let (canonical, productive) = fixture_owners();
    CanonicalL1AnchorReplayContextV1::new(
        canonical,
        productive.l11_package_sha256(),
        productive.canonical_l2_package_sha256(),
    )
}

fn exact_peaks(source: &str, surfaces: &[&str]) -> ExactPeakBirthEnumerationV1 {
    let canonical = &fixture_owners().0;
    let rows = surfaces
        .iter()
        .map(|surface| {
            (
                canonical
                    .form_ref_for_surface(surface)
                    .expect("fixture surface must have a canonical form ref"),
                *surface,
            )
        })
        .collect::<Vec<_>>();
    exact_peaks_with_rows(source, &rows)
}

fn exact_peaks_with_rows(source: &str, rows: &[(u32, &str)]) -> ExactPeakBirthEnumerationV1 {
    let oracle = Phase7dCertificateOracle::new(source).expect("typed geometry oracle");
    ExactPeakBirthEnumerationV1::from_candidates(
        rows.iter()
            .map(|(form_ref, surface)| ExactPeakCandidateInputV1 {
                form_ref: *form_ref,
                normalized_surface: (*surface).to_string(),
                certificates: oracle
                    .certificate_evidence(surface)
                    .expect("exact peak certificate"),
            })
            .collect(),
    )
    .expect("exact peak enumeration")
    .with_test_search_proof(source, fixture_owners().1.canonical_l2_package_sha256())
    .expect("completed exact search proof")
}

pub(in crate::nanda_wave::l2_field::productive_v1) fn raw_peaks_with_unsupported_overflow(
    source: &str,
    supported_surfaces: &[&str],
) -> ExactPeakBirthEnumerationV1 {
    use crate::typing_transition::target_evidence::MAX_TARGETS_PER_FIELD;

    let oracle = Phase7dCertificateOracle::new(source).expect("fixture oracle");
    let canonical = &fixture_owners().0;
    let mut inputs = supported_surfaces
        .iter()
        .enumerate()
        .map(|(index, surface)| ExactPeakCandidateInputV1 {
            form_ref: canonical
                .form_ref_for_surface(surface)
                .unwrap_or(10_000 + index as u32),
            normalized_surface: (*surface).to_string(),
            certificates: oracle
                .certificate_evidence(surface)
                .expect("fixture evidence"),
        })
        .collect::<Vec<_>>();
    let original = source.chars().collect::<Vec<_>>();
    assert!(original.len() >= 2);
    let mut unsupported_count = 0;
    'variants: for first in 'а'..='я' {
        for second in 'а'..='я' {
            if first == original[0] || second == original[1] {
                continue;
            }
            let mut symbols = original.clone();
            symbols[0] = first;
            symbols[1] = second;
            let surface = symbols.iter().collect::<String>();
            let certificates = oracle
                .certificate_evidence(&surface)
                .expect("typed evidence");
            if certificates.is_empty()
                || certificates
                    .iter()
                    .any(|evidence| supported_phase7d_relation(evidence).is_some())
            {
                continue;
            }
            inputs.push(ExactPeakCandidateInputV1 {
                form_ref: 1_000 + unsupported_count as u32,
                normalized_surface: surface,
                certificates,
            });
            unsupported_count += 1;
            if unsupported_count > MAX_TARGETS_PER_FIELD {
                break 'variants;
            }
        }
    }
    assert_eq!(unsupported_count, MAX_TARGETS_PER_FIELD + 1);
    ExactPeakBirthEnumerationV1::from_candidates(inputs).expect("complete raw fixture")
}

#[test]
fn raw_exact_overflow_preserves_bounded_partition_and_common_incompleteness() {
    use crate::typing_transition::target_evidence::{EnumerationStateV1, MAX_TARGETS_PER_FIELD};

    for targets in [vec!["форма"], vec!["форма", "формы"], Vec::new()] {
        let raw = raw_peaks_with_unsupported_overflow(OBSERVED, &targets);
        let diagnostic = raw.diagnostic_json();
        let raw_count = MAX_TARGETS_PER_FIELD + 1 + targets.len();
        assert!(raw.capacity_exceeded());
        assert_eq!(diagnostic["candidate_count"], raw_count);
        let peaks = raw
            .with_test_search_proof(OBSERVED, fixture_owners().1.canonical_l2_package_sha256())
            .expect("raw storage overflow must not discard a complete bounded authority partition");
        assert_eq!(
            peaks.diagnostic_json(),
            diagnostic,
            "raw evidence was narrowed"
        );
        let field = prepared_field(OBSERVED, peaks);
        assert_eq!(field.replacement_lattice_surfaces(), targets);
        for completeness in [
            field.common_completeness(),
            field.prepared_material().completeness(),
        ] {
            assert_eq!(completeness.state(), EnumerationStateV1::Overflow);
            assert_eq!(
                completeness.reason(),
                IncompletenessReasonV1::StorageCapacity
            );
            assert_eq!(usize::from(completeness.retained_count()), targets.len());
            assert_eq!(
                usize::from(completeness.logical_count_lower_bound()),
                raw_count
            );
        }
        assert!(field.authority_partition_valid_for_test());
        assert_eq!(
            field.authority_material().completeness().state(),
            EnumerationStateV1::Complete
        );
        let mut candidates =
            super::super::live::materialize_live_productive_v1_field(ORIGINAL, OBSERVED, &field)
                .expect("bounded materialization")
                .candidates;
        assert_eq!(
            candidates
                .iter()
                .map(|value| value.replacement.as_str())
                .collect::<Vec<_>>(),
            targets
                .iter()
                .map(|target| format!("{CONTEXT}{target} "))
                .collect::<Vec<_>>()
        );
        assert!(candidates
            .iter()
            .all(|value| value.gate.action != CandidateGateAction::Eligible));
        let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
        let mut settled = settle_shared_canonical_cohort(
            Arc::clone(&field),
            Some(&frame),
            current_generation(),
            ORIGINAL,
            anchor_replay(),
        );
        let singleton = targets.len() == 1;
        assert_eq!(settled.capability_count(), usize::from(singleton));
        project_frame_bound_lexical_capability(&mut candidates, &mut settled.authority_context);
        let event = event(ORIGINAL, TypingErrorClass::MissingLetter);
        assert_eq!(
            settled
                .authority_context
                .admissions_for_event(&event, &candidates),
            vec![singleton; targets.len()]
        );
        if singleton {
            assert_eq!(candidates[0].replacement, WINNER);
            assert!(candidates[0].frame_bound_lexical_capability().is_some());
        } else {
            assert!(candidates
                .iter()
                .all(|value| value.frame_bound_lexical_capability().is_none()));
        }
        for safety in [
            CorrectionSafety::Normal,
            CorrectionSafety::Strict,
            CorrectionSafety::Experimental,
        ] {
            let evaluated = TransitionDecisionCore::evaluate_candidates_with_authority_context(
                &event,
                &candidates,
                TransitionDecisionPolicy {
                    l2_phase_apply: false,
                    correction_safety: safety,
                },
                DecisionEvidenceMode::FullField(None),
                &settled.authority_context,
            );
            assert_eq!(
                evaluated.selected_index,
                (singleton && safety != CorrectionSafety::Strict).then_some(0),
                "{targets:?} {safety:?}"
            );
            if singleton {
                assert!(evaluated.evaluations[0].action.verifier_passed);
            }
        }
    }
}

#[test]
fn bounded_partition_after_raw_overflow_still_requires_the_current_frame() {
    let raw = raw_peaks_with_unsupported_overflow(OBSERVED, &["форма"])
        .with_test_search_proof(OBSERVED, fixture_owners().1.canonical_l2_package_sha256())
        .expect("bounded supported singleton");
    let field = prepared_field(OBSERVED, raw);
    let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
    for (current, expected) in [(Some(&frame), true), (None, false)] {
        let mut settled = settle_shared_canonical_cohort(
            Arc::clone(&field),
            current,
            current_generation(),
            ORIGINAL,
            anchor_replay(),
        );
        assert_eq!(settled.capability_count(), usize::from(expected));
        let mut candidates = vec![candidate(WINNER, TypingErrorClass::MissingLetter)];
        project_frame_bound_lexical_capability(&mut candidates, &mut settled.authority_context);
        let event = event(ORIGINAL, TypingErrorClass::MissingLetter);
        assert_eq!(
            settled
                .authority_context
                .admissions_for_event(&event, &candidates),
            vec![expected]
        );
        assert_eq!(
            candidates[0].frame_bound_lexical_capability().is_some(),
            expected
        );
        if !expected {
            let evaluated = TransitionDecisionCore::evaluate_candidates_with_authority_context(
                &event,
                &candidates,
                TransitionDecisionPolicy {
                    l2_phase_apply: false,
                    correction_safety: CorrectionSafety::Experimental,
                },
                DecisionEvidenceMode::FullField(None),
                &settled.authority_context,
            );
            assert_eq!(evaluated.selected_index, None);
            assert!(evaluated.evaluations[0].action.verifier_passed);
        }
    }

    let mut settled = settle_shared_canonical_cohort(
        field,
        Some(&frame),
        current_generation(),
        ORIGINAL,
        anchor_replay(),
    );
    let mut candidates = vec![candidate(WINNER, TypingErrorClass::MissingLetter)];
    project_frame_bound_lexical_capability(&mut candidates, &mut settled.authority_context);
    let LexicalAuthorityEvaluationContextV1::Certified(bound) = &settled.authority_context else {
        panic!("positive issuer control must certify");
    };
    let stale = LexicalAuthorityEvaluationContextV1::Certified(BoundSettlementContextV1 {
        settlement: Arc::clone(&bound.settlement),
        current_frame: lexical_frame("другая ", OBSERVED, "другая ", OBSERVED),
    });
    let event = event(ORIGINAL, TypingErrorClass::MissingLetter);
    assert_eq!(
        settled
            .authority_context
            .admissions_for_event(&event, &candidates),
        vec![true]
    );
    assert_eq!(stale.admissions_for_event(&event, &candidates), vec![false]);
    assert!(candidates[0].frame_bound_lexical_capability().is_some());
    assert!(!settled.authority_context.admits_candidate_at(
        &event,
        &candidates[0],
        bound
            .settlement
            .lease
            .expires_at_monotonic_ns
            .saturating_add(1),
    ));
    let evaluated = TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        &candidates,
        TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        DecisionEvidenceMode::FullField(None),
        &stale,
    );
    assert_eq!(evaluated.selected_index, None);
    assert!(evaluated.evaluations[0].action.verifier_passed);
}

fn wrong_geometry_peak(source: &str, surface: &str) -> ExactPeakBirthEnumerationV1 {
    exact_peaks(source, &[surface])
        .with_corrupted_first_class_for_test(Phase7dCertificateClass::ExtraLetter)
}

fn prepared_field(
    source: &str,
    peaks: ExactPeakBirthEnumerationV1,
) -> Arc<PreparedCanonicalTokenField> {
    let (canonical, productive) = fixture_owners();
    Arc::new(
        super::super::live::prepare_live_productive_v1_field_with_exact_peaks(
            CONTEXT,
            source,
            canonical,
            productive,
            &[],
            &[],
            &[],
            peaks,
        )
        .expect("prepared fixture field"),
    )
}

fn prepared_grounded_field(
    source: &str,
    target: &str,
    peaks: ExactPeakBirthEnumerationV1,
) -> Arc<PreparedCanonicalTokenField> {
    let (canonical, productive) = fixture_owners();
    let contour = CanonicalContourSeed {
        query_surface: source.to_string(),
        seed: L11SeedSurface {
            terminal_id: Some(17),
            surface: target.to_string(),
            authority: false,
            score_milli: 1_000,
        },
        relation: CanonicalContourRelation::Identity,
    };
    Arc::new(
        super::super::live::prepare_live_productive_v1_field_with_exact_peaks(
            CONTEXT,
            source,
            canonical,
            productive,
            std::slice::from_ref(&contour),
            &[],
            &[],
            peaks,
        )
        .expect("prepared grounded fixture field"),
    )
}

fn lexical_frame(
    context: &str,
    source: &str,
    coordinate_context: &str,
    coordinate_source: &str,
) -> LexicalAuthorityFrameV1 {
    let config = crate::config::LayConfig::default();
    let config_identity =
        crate::lexical_authority_frame::LexicalAuthorityConfigIdentityV1::from_config(&config);
    let coordinate_scalars = coordinate_source.chars().count() as u32;
    let coordinates = crate::lexical_authority_frame::LexicalAuthorityCoordinatesV1::new(
        41,
        [41, 42],
        43,
        coordinate_source.to_string(),
        coordinate_context.to_string(),
        coordinate_scalars,
        (coordinate_scalars, coordinate_scalars),
        String::new(),
        0,
        44,
        config_identity.identity_fingerprint(),
    )
    .expect("valid fixture coordinates");
    LexicalAuthorityFrameV1::from_exact_parts(
        "/test/td117-boundary".to_string(),
        Some("focus".to_string()),
        42,
        format!("{context}{source}"),
        context.to_string(),
        source.to_string(),
        false,
        true,
        crate::exact_layout_authority::FactoryEngineProfile::Ru,
        None,
        1,
        2,
        config_identity,
    )
    .with_coordinates(Some(coordinates))
}

fn current_generation() -> u64 {
    super::super::super::cache::stats().generation
}

fn settle(
    source: &str,
    original: &str,
    peaks: ExactPeakBirthEnumerationV1,
    frame: &LexicalAuthorityFrameV1,
) -> LexicalCohortSettlementReadoutV1 {
    settle_shared_canonical_cohort(
        prepared_field(source, peaks),
        Some(frame),
        current_generation(),
        original,
        anchor_replay(),
    )
}

fn settle_grounded(
    source: &str,
    target: &str,
    original: &str,
    peaks: ExactPeakBirthEnumerationV1,
    frame: &LexicalAuthorityFrameV1,
) -> LexicalCohortSettlementReadoutV1 {
    settle_shared_canonical_cohort(
        prepared_grounded_field(source, target, peaks),
        Some(frame),
        current_generation(),
        original,
        anchor_replay(),
    )
}

fn event(original: &str, error_class: TypingErrorClass) -> TypingErrorEvent {
    TypingErrorEvent {
        original: original.to_string(),
        core: original.trim().to_string(),
        current_word: original
            .split_whitespace()
            .last()
            .unwrap_or_default()
            .to_string(),
        input_class: error_class,
    }
}

fn candidate(replacement: &str, error_class: TypingErrorClass) -> UnifiedCorrectionCandidate {
    UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
        CandidateOrigin::L2Surface,
        crate::nanda_wave::l2_field::CANONICAL_L2_SURFACE_SOURCE_ID,
        error_class,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "td117_boundary_fixture",
        },
    )
    .with_canonical_l2_exact_partition_membership()
}

pub(crate) fn valid_pipeline(
    source: &str,
    target: &str,
    error_class: TypingErrorClass,
) -> (
    TypingErrorEvent,
    UnifiedCorrectionCandidate,
    LexicalAuthorityEvaluationContextV1,
) {
    let original = format!("{CONTEXT}{source} ");
    let replacement = format!("{CONTEXT}{target} ");
    let frame = lexical_frame(CONTEXT, source, CONTEXT, source);
    let settled = settle(source, &original, exact_peaks(source, &[target]), &frame);
    assert!(
        settled.authority_context.is_certified(),
        "grounded singleton fixture must certify"
    );
    let mut authority_context = settled.authority_context;
    let mut candidates = vec![candidate(&replacement, error_class)];
    project_frame_bound_lexical_capability(&mut candidates, &mut authority_context);
    assert!(
        authority_context.is_certified(),
        "unique matching projection must preserve certification"
    );
    assert!(candidates[0].frame_bound_lexical_capability().is_some());
    (
        event(&original, error_class),
        candidates.remove(0),
        authority_context,
    )
}

#[test]
fn td117_canonical_l1_lemma_anchor_requires_exact_partition_form_parity() {
    let canonical = &fixture_owners().0;
    let target_form_ref = canonical
        .form_ref_for_surface("форма")
        .expect("bound target form ref");
    let other_form_ref = canonical
        .form_ref_for_surface("формы")
        .expect("other target form ref");
    let unbound_form_ref = canonical
        .form_ref_for_surface("форму")
        .expect("unbound target form ref");
    let unknown_form_ref = u32::MAX;
    let ambiguous_form_ref = canonical
        .form_ref_for_surface("общую")
        .expect("multi-lemma target form ref");
    assert_eq!(
        canonical.decode_form_ref(target_form_ref).as_deref(),
        Some("форма")
    );
    assert_eq!(
        canonical.l1_terminal_for_form_ref(target_form_ref),
        Some(17)
    );
    assert_eq!(canonical.l1_terminal_for_form_ref(unbound_form_ref), None);
    let direct_anchor = canonical
        .l1_lexical_anchor_for_form_ref(target_form_ref)
        .expect("direct target terminal is a lexical anchor");
    assert!(direct_anchor.is_direct());
    assert_eq!(direct_anchor.anchor_form_ref(), target_form_ref);
    assert_eq!(direct_anchor.terminal_id(), 17);
    let lemma_anchor = canonical
        .l1_lexical_anchor_for_form_ref(unbound_form_ref)
        .expect("same-lemma L1 terminal anchors a materialized form");
    assert!(!lemma_anchor.is_direct());
    assert_eq!(lemma_anchor.anchor_form_ref(), target_form_ref);
    assert_eq!(lemma_anchor.terminal_id(), 17);
    assert_eq!(
        canonical.l1_terminal_for_form_ref(lemma_anchor.anchor_form_ref()),
        Some(lemma_anchor.terminal_id())
    );
    assert!(lemma_anchor.lemma_id().is_some());
    assert_eq!(canonical.decode_form_ref(unknown_form_ref), None);
    assert_eq!(
        canonical.l1_lexical_anchor_for_form_ref(unknown_form_ref),
        None
    );
    assert_eq!(
        canonical.l1_lexical_anchor_for_form_ref(ambiguous_form_ref),
        None
    );

    let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
    let positive = settle(
        OBSERVED,
        ORIGINAL,
        exact_peaks(OBSERVED, &["форма"]),
        &frame,
    );
    assert!(positive.authority_context.is_certified());
    assert_eq!(positive.capability_count(), 1);

    let mismatched_form = settle(
        OBSERVED,
        ORIGINAL,
        exact_peaks_with_rows(OBSERVED, &[(other_form_ref, "форма")]),
        &frame,
    );
    assert_projection_stays_empty(mismatched_form, WINNER);

    let lemma_anchored_target = settle(
        OBSERVED,
        ORIGINAL,
        exact_peaks_with_rows(OBSERVED, &[(unbound_form_ref, "форму")]),
        &frame,
    );
    assert!(lemma_anchored_target.authority_context.is_certified());
    assert_eq!(lemma_anchored_target.capability_count(), 1);

    let unknown_target = settle(
        OBSERVED,
        ORIGINAL,
        exact_peaks_with_rows(OBSERVED, &[(unknown_form_ref, "ферм")]),
        &frame,
    );
    assert_projection_stays_empty(unknown_target, "нужна ферм ");

    let mut l11_only_surface = settle_grounded(
        OBSERVED,
        "форма",
        ORIGINAL,
        exact_peaks_with_rows(OBSERVED, &[(unbound_form_ref, "форму")]),
        &frame,
    );
    assert!(l11_only_surface.authority_context.is_certified());
    assert_eq!(l11_only_surface.winner_replacement(), Some("нужна форму "));
    let mut l11_only_candidate = vec![candidate(WINNER, TypingErrorClass::MissingLetter)];
    project_frame_bound_lexical_capability(
        &mut l11_only_candidate,
        &mut l11_only_surface.authority_context,
    );
    assert!(l11_only_candidate[0]
        .frame_bound_lexical_capability()
        .is_none());
    assert!(!l11_only_surface.authority_context.is_certified());
}

#[test]
fn td117_canonical_l1_anchor_tamper_matrix_fails_closed_after_reseal() {
    let target = "форму";
    let original = ORIGINAL;
    let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
    for (dimension, fault, structurally_resealed) in [
        (
            "target-bytes",
            CanonicalL1AnchorProofFaultV1::TargetBytes,
            false,
        ),
        (
            "target-form",
            CanonicalL1AnchorProofFaultV1::TargetForm,
            true,
        ),
        (
            "anchor-form",
            CanonicalL1AnchorProofFaultV1::AnchorForm,
            true,
        ),
        ("lemma", CanonicalL1AnchorProofFaultV1::Lemma, true),
        ("terminal", CanonicalL1AnchorProofFaultV1::Terminal, true),
        ("kind", CanonicalL1AnchorProofFaultV1::Kind, false),
        (
            "l11-package",
            CanonicalL1AnchorProofFaultV1::L11Package,
            false,
        ),
        (
            "canonical-l2-package",
            CanonicalL1AnchorProofFaultV1::CanonicalL2Package,
            false,
        ),
    ] {
        let mut field = Arc::try_unwrap(prepared_field(OBSERVED, exact_peaks(OBSERVED, &[target])))
            .expect("unshared test field");
        field.corrupt_canonical_l1_anchor_proof_for_test(fault);
        assert_eq!(
            field.authority_partition_valid_for_test(),
            structurally_resealed,
            "unexpected inner proof state for {dimension}",
        );
        let settled = settle_shared_canonical_cohort(
            Arc::new(field),
            Some(&frame),
            current_generation(),
            original,
            anchor_replay(),
        );
        assert_eq!(settled.capability_count(), 0, "tampered {dimension}");
        assert!(
            !settled.authority_context.is_certified(),
            "tampered {dimension} retained authority",
        );
    }
}

#[test]
fn td117_canonical_l1_anchor_missing_extra_ambiguous_or_raw_terminal_fails_closed() {
    let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
    for (case, fault) in [
        ("missing", CanonicalL1AnchorProofFaultV1::Missing),
        ("extra", CanonicalL1AnchorProofFaultV1::Extra),
        ("ambiguous", CanonicalL1AnchorProofFaultV1::Ambiguous),
        (
            "raw-terminal",
            CanonicalL1AnchorProofFaultV1::RawTerminalSubstitution,
        ),
    ] {
        let mut field =
            Arc::try_unwrap(prepared_field(OBSERVED, exact_peaks(OBSERVED, &["форму"])))
                .expect("unshared test field");
        field.corrupt_canonical_l1_anchor_proof_for_test(fault);
        assert!(!field.authority_partition_valid_for_test(), "{case}");
        let settled = settle_shared_canonical_cohort(
            Arc::new(field),
            Some(&frame),
            current_generation(),
            ORIGINAL,
            anchor_replay(),
        );
        assert_eq!(settled.capability_count(), 0, "{case}");
        assert!(!settled.authority_context.is_certified(), "{case}");
    }
}

#[test]
fn td117_canonical_materialized_original_uses_same_anchor_and_is_preserved() {
    let source = "форму";
    let original = "нужна форму ";
    let frame = lexical_frame(CONTEXT, source, CONTEXT, source);
    let settled = settle(source, original, exact_peaks(source, &["форма"]), &frame);

    assert_projection_stays_empty(settled, WINNER);
}

#[test]
fn td117_observed_same_lemma_preservation_requires_consumer_anchor_replay() {
    let source = "форму";
    let original = "нужна форму ";
    let frame = lexical_frame(CONTEXT, source, CONTEXT, source);
    let mut field = Arc::try_unwrap(prepared_field(source, exact_peaks(source, &["форма"])))
        .expect("unshared observed-anchor field");
    field.corrupt_observed_canonical_l1_anchor_for_test();
    assert!(field.authority_partition_valid_for_test());

    let settled = settle_shared_canonical_cohort(
        Arc::new(field),
        Some(&frame),
        current_generation(),
        original,
        anchor_replay(),
    );

    assert_eq!(settled.capability_count(), 0);
    assert!(!settled.authority_context.is_certified());
    assert_eq!(
        settled.compare.status,
        CohortCompareStatusV1::SettlementFailed
    );
}

#[test]
fn td117_equivalent_supported_roots_for_one_target_are_not_a_target_tie() {
    let source = "каса";
    let target = "касса";
    let original = "нужна каса ";
    let frame = lexical_frame(CONTEXT, source, CONTEXT, source);
    let peaks = exact_peaks(source, &[target]);
    let diagnostic = peaks.diagnostic_json();
    assert_eq!(diagnostic["candidate_count"], 1);
    assert_eq!(diagnostic["certificate_count"], 2);

    let settled = settle(source, original, peaks, &frame);
    assert!(settled.authority_context.is_certified());
    assert_eq!(settled.capability_count(), 1);
}

fn assert_projection_stays_empty(mut settled: LexicalCohortSettlementReadoutV1, replacement: &str) {
    assert!(
        !settled.authority_context.is_certified(),
        "unexpected certified settlement for {replacement:?}: winner={:?}",
        settled.winner_replacement()
    );
    assert_eq!(settled.capability_count(), 0);
    let mut candidates = vec![candidate(replacement, TypingErrorClass::MissingLetter)];
    project_frame_bound_lexical_capability(&mut candidates, &mut settled.authority_context);
    assert!(candidates[0].frame_bound_lexical_capability().is_none());
    assert_eq!(
        settled.authority_context.admissions_for_event(
            &event(ORIGINAL, TypingErrorClass::MissingLetter),
            &candidates
        ),
        vec![false]
    );
}

#[test]
fn td117_issuer_projection_consumer_invalid_issuers_never_project_authority() {
    let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
    let stale_frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, "форма");
    for (name, settled) in [
        (
            "tied",
            settle(
                OBSERVED,
                ORIGINAL,
                exact_peaks(OBSERVED, &["форма", "формы"]),
                &frame,
            ),
        ),
        (
            "upstream-incomplete",
            settle(
                OBSERVED,
                ORIGINAL,
                ExactPeakBirthEnumerationV1::incomplete(IncompletenessReasonV1::UpstreamIncomplete),
                &frame,
            ),
        ),
        (
            "stale-frame",
            settle(
                OBSERVED,
                ORIGINAL,
                exact_peaks(OBSERVED, &["форма"]),
                &stale_frame,
            ),
        ),
        (
            "wrong-geometry",
            settle(
                OBSERVED,
                ORIGINAL,
                wrong_geometry_peak(OBSERVED, "форма"),
                &frame,
            ),
        ),
    ] {
        if name == "stale-frame" {
            assert!(matches!(
                settled.authority_context,
                LexicalAuthorityEvaluationContextV1::Invalid(
                    LexicalAuthorityFailureV1::FrameMismatch
                )
            ));
        }
        assert_projection_stays_empty(settled, WINNER);
    }
}

#[test]
fn td117_issuer_projection_consumer_projection_and_merge_fail_closed() {
    #[derive(Clone, Copy)]
    enum Case {
        ZeroMatches,
        DuplicateMatches,
        ForeignCapability,
    }

    for case in [
        Case::ZeroMatches,
        Case::DuplicateMatches,
        Case::ForeignCapability,
    ] {
        let (event, mut candidates, mut authority_context, expected_failure) = match case {
            Case::ZeroMatches | Case::DuplicateMatches => {
                let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
                let settled = settle_grounded(
                    OBSERVED,
                    "форма",
                    ORIGINAL,
                    exact_peaks(OBSERVED, &["форма"]),
                    &frame,
                );
                assert!(settled.authority_context.is_certified());
                let candidates = match case {
                    Case::ZeroMatches => Vec::new(),
                    Case::DuplicateMatches => vec![
                        candidate(WINNER, TypingErrorClass::MissingLetter),
                        candidate(WINNER, TypingErrorClass::MissingLetter),
                    ],
                    Case::ForeignCapability => unreachable!(),
                };
                (
                    event(ORIGINAL, TypingErrorClass::MissingLetter),
                    candidates,
                    settled.authority_context,
                    LexicalAuthorityFailureV1::ProjectionCardinality,
                )
            }
            Case::ForeignCapability => {
                let (event, mut local, local_context) =
                    valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
                let (_, foreign, foreign_context) =
                    valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
                assert!(
                    local_context.admissions_for_event(&event, std::slice::from_ref(&local))[0]
                );
                assert!(
                    foreign_context.admissions_for_event(&event, std::slice::from_ref(&foreign))[0]
                );
                local.merge_evidence(foreign);
                assert!(local.has_authority_conflict());
                (
                    event,
                    vec![local],
                    local_context,
                    LexicalAuthorityFailureV1::AuthorityConflict,
                )
            }
        };
        project_frame_bound_lexical_capability(&mut candidates, &mut authority_context);
        let actual_failure = match &authority_context {
            LexicalAuthorityEvaluationContextV1::InvalidAfterCertification(_, failure) => *failure,
            other => panic!("expected sticky projection failure, got {other:#?}"),
        };
        assert_eq!(actual_failure, expected_failure);
        assert!(authority_context.owns_canonical_l2_field());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.frame_bound_lexical_capability().is_none()));

        let probe = candidate(WINNER, TypingErrorClass::MissingLetter);
        assert_eq!(
            authority_context.admissions_for_event(&event, std::slice::from_ref(&probe),),
            vec![false]
        );
        let batch = TransitionDecisionCore::evaluate_candidates_with_authority_context(
            &event,
            std::slice::from_ref(&probe),
            TransitionDecisionPolicy {
                l2_phase_apply: false,
                correction_safety: CorrectionSafety::Experimental,
            },
            DecisionEvidenceMode::FullField(None),
            &authority_context,
        );
        assert_eq!(batch.selected_index, None, "{batch:#?}");
    }
}

#[test]
fn td117_projection_rejects_same_bytes_without_typed_exact_partition_lane() {
    let frame = lexical_frame(CONTEXT, OBSERVED, CONTEXT, OBSERVED);
    for (name, untyped) in [
        (
            "deterministic",
            UnifiedCorrectionCandidate::new(
                WINNER,
                CorrectionDecisionSource::Deterministic,
                CandidateOrigin::DeterministicTypo,
                "missing_letter",
                TypingErrorClass::MissingLetter,
                CandidateGateDecision {
                    action: CandidateGateAction::Eligible,
                    reason: "independent_deterministic_lane",
                },
            ),
        ),
        (
            "typed-exact-source-id-only",
            UnifiedCorrectionCandidate::new(
                WINNER,
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::L2Surface,
                super::super::live::PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID,
                TypingErrorClass::MissingLetter,
                CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "diagnostic_source_id_only",
                },
            ),
        ),
        (
            "legacy-canonical-without-exact-membership",
            UnifiedCorrectionCandidate::new(
                WINNER,
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::L2Surface,
                crate::nanda_wave::l2_field::CANONICAL_L2_SURFACE_SOURCE_ID,
                TypingErrorClass::MissingLetter,
                CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "legacy_membership_only",
                },
            ),
        ),
    ] {
        let settled = settle_grounded(
            OBSERVED,
            "форма",
            ORIGINAL,
            exact_peaks(OBSERVED, &["форма"]),
            &frame,
        );
        assert!(settled.authority_context.is_certified(), "{name}");
        let mut authority_context = settled.authority_context;
        let mut candidates = vec![untyped];

        project_frame_bound_lexical_capability(&mut candidates, &mut authority_context);

        assert!(
            candidates[0].frame_bound_lexical_capability().is_none(),
            "{name}"
        );
        assert!(
            matches!(
                authority_context,
                LexicalAuthorityEvaluationContextV1::InvalidAfterCertification(
                    _,
                    LexicalAuthorityFailureV1::ProjectionCardinality
                )
            ),
            "{name}"
        );
    }
}

#[test]
fn td117_failed_lexical_certification_preserves_independent_deterministic_lane() {
    let (event, mut merged, authority_context) =
        valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
    merged.merge_evidence(UnifiedCorrectionCandidate::new(
        WINNER,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "missing_letter",
        TypingErrorClass::MissingLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "independent_deterministic_lane",
        },
    ));
    let failed_context = match authority_context {
        LexicalAuthorityEvaluationContextV1::Certified(context) => {
            LexicalAuthorityEvaluationContextV1::InvalidAfterCertification(
                context,
                LexicalAuthorityFailureV1::ProjectionCardinality,
            )
        }
        other => panic!("expected certified context, got {other:#?}"),
    };

    let batch = TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        std::slice::from_ref(&merged),
        TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        DecisionEvidenceMode::FullField(None),
        &failed_context,
    );

    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
}

#[test]
fn td117_foreign_lexical_conflict_invalidates_only_the_canonical_lane() {
    let (event, mut merged, authority_context) =
        valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
    let (_, foreign, _) = valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
    merged.merge_evidence(foreign);
    assert!(merged.has_authority_conflict());
    merged.merge_evidence(UnifiedCorrectionCandidate::new(
        WINNER,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::DeterministicTypo,
        "missing_letter",
        TypingErrorClass::MissingLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "independent_deterministic_lane",
        },
    ));

    assert_eq!(
        authority_context.admissions_for_event(&event, std::slice::from_ref(&merged)),
        vec![false]
    );
    let batch = TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        std::slice::from_ref(&merged),
        TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        DecisionEvidenceMode::FullField(None),
        &authority_context,
    );
    assert_eq!(batch.selected_index, Some(0), "{batch:#?}");
}

#[test]
fn td117_post_projection_alias_merge_preserves_the_canonical_owner_lane() {
    let (event, canonical, authority_context) =
        valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
    let mut ordinary = UnifiedCorrectionCandidate::new(
        WINNER,
        CorrectionDecisionSource::Deterministic,
        CandidateOrigin::Boundary,
        "same_surface_boundary",
        TypingErrorClass::MissingLetter,
        CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "independent_boundary_lane",
        },
    );
    ordinary.merge_evidence(canonical);

    let canonical_lane = ordinary
        .frame_bound_lexical_lane_view()
        .expect("merged surface retains its exact canonical owner lane");
    assert_eq!(canonical_lane.origin, CandidateOrigin::L2Surface);
    assert!(authority_context.admits_candidate_at(
        &event,
        &ordinary,
        match &authority_context {
            LexicalAuthorityEvaluationContextV1::Certified(bound) => {
                bound.settlement.lease.expires_at_monotonic_ns
            }
            other => panic!("expected certified context, got {other:#?}"),
        }
    ));
}

#[test]
fn td117_issuer_projection_consumer_rejects_changed_event_frame_or_expiry() {
    let (event, projected, authority_context) =
        valid_pipeline(OBSERVED, "форма", TypingErrorClass::MissingLetter);
    let bound = match &authority_context {
        LexicalAuthorityEvaluationContextV1::Certified(bound) => bound,
        other => panic!("expected certified context, got {other:#?}"),
    };
    let expires_at = bound.settlement.lease.expires_at_monotonic_ns;
    assert!(authority_context.admits_candidate_at(&event, &projected, expires_at));

    let mut changed_event = event.clone();
    changed_event.original = "другая форм ".to_string();
    assert!(!authority_context.admits_candidate_at(&changed_event, &projected, expires_at));

    let changed_frame_context =
        LexicalAuthorityEvaluationContextV1::Certified(BoundSettlementContextV1 {
            settlement: Arc::clone(&bound.settlement),
            current_frame: lexical_frame("другая ", OBSERVED, "другая ", OBSERVED),
        });
    assert!(!changed_frame_context.admits_candidate_at(&event, &projected, expires_at));
    assert!(!authority_context.admits_candidate_at(
        &event,
        &projected,
        expires_at.saturating_add(1)
    ));
    assert!(projected.frame_bound_lexical_capability().is_some());
}

#[test]
fn td117_issuer_projection_consumer_bound_capability_cannot_bypass_safety() {
    let (event, projected, authority_context) =
        valid_pipeline(OBSERVED, "форма", TypingErrorClass::LetterSubstitution);
    assert!(authority_context.admissions_for_event(&event, std::slice::from_ref(&projected))[0]);

    let experimental = TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        std::slice::from_ref(&projected),
        TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Experimental,
        },
        DecisionEvidenceMode::FullField(None),
        &authority_context,
    );
    // Experimental explicitly requires zero additional evidence domains, so a
    // verifier-bound lexical capability may be selected under that profile.
    assert_eq!(experimental.selected_index, Some(0));

    let strict = TransitionDecisionCore::evaluate_candidates_with_authority_context(
        &event,
        std::slice::from_ref(&projected),
        TransitionDecisionPolicy {
            l2_phase_apply: false,
            correction_safety: CorrectionSafety::Strict,
        },
        DecisionEvidenceMode::FullField(None),
        &authority_context,
    );
    // Strict still requires two independent domains. The lexical capability
    // contributes one verified L2 domain and cannot bypass that policy.
    assert_eq!(strict.selected_index, None);
}
