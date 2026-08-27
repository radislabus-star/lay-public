use super::*;

#[derive(Clone, Copy)]
struct AdmissionFixture {
    original: &'static str,
    replacement: &'static str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
}

const EXISTING_ADMISSION_FIXTURES: &[AdmissionFixture] = &[
    AdmissionFixture {
        original: "40 000 р ",
        replacement: "40 000 h ",
        error_class: TypingErrorClass::WrongLayout,
        origin: CandidateOrigin::DeterministicTypo,
    },
    AdmissionFixture {
        original: "Екб ",
        replacement: "Tr, ",
        error_class: TypingErrorClass::WrongLayout,
        origin: CandidateOrigin::DeterministicTypo,
    },
    AdmissionFixture {
        original: "дфн ",
        replacement: "lay ",
        error_class: TypingErrorClass::WrongLayout,
        origin: CandidateOrigin::Layout,
    },
    AdmissionFixture {
        original: "в коде ",
        replacement: "в код ",
        error_class: TypingErrorClass::ExtraLetter,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "закинем ",
        replacement: "закон ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "китайцы ",
        replacement: "китайы ",
        error_class: TypingErrorClass::ExtraLetter,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "ходу ",
        replacement: "ход ",
        error_class: TypingErrorClass::ExtraLetter,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "делаем ",
        replacement: "деваем ",
        error_class: TypingErrorClass::LetterSubstitution,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "допусти мнабираю ",
        replacement: "допустим набираю ",
        error_class: TypingErrorClass::BoundaryShift,
        origin: CandidateOrigin::Boundary,
    },
    AdmissionFixture {
        original: "я думаю допусти мнабираю ",
        replacement: "я думаю допустим набираю ",
        error_class: TypingErrorClass::BoundaryShift,
        origin: CandidateOrigin::Boundary,
    },
    AdmissionFixture {
        original: "тоесть ",
        replacement: "есть ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "что получилось содержкой ",
        replacement: "что получилось содержать ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "патерна ",
        replacement: "пара ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L3Context,
    },
    AdmissionFixture {
        original: "я прохоил ",
        replacement: "я проход ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "ответили вчате ",
        replacement: "ответили вате ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L3Context,
    },
    AdmissionFixture {
        original: "будет примать ",
        replacement: "будет придать ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L3Context,
    },
    AdmissionFixture {
        original: "видешь ",
        replacement: "видишь ",
        error_class: TypingErrorClass::LetterSubstitution,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "дожь ",
        replacement: "дождь ",
        error_class: TypingErrorClass::MissingLetter,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "твой ",
        replacement: "тывой ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::DeterministicTypo,
    },
    AdmissionFixture {
        original: "что нравится? ",
        replacement: "что нравиться? ",
        error_class: TypingErrorClass::MissingLetter,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "Читал логи ",
        replacement: "Читал логик ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "смотри, ",
        replacement: "смотори, ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::DeterministicTypo,
    },
    AdmissionFixture {
        original: "давай там посмотри ",
        replacement: "давай там просмотри ",
        error_class: TypingErrorClass::MissingLetter,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "посмотри ",
        replacement: "посмотреть ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "искать хрень! ",
        replacement: "искать хрену ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::DeterministicTypo,
    },
    AdmissionFixture {
        original: "тели ",
        replacement: "тел ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
    AdmissionFixture {
        original: "нас моного ",
        replacement: "нас мюоного ",
        error_class: TypingErrorClass::CompositeTypo,
        origin: CandidateOrigin::L2Surface,
    },
];

#[test]
fn lexical_fact_reuse_preserves_existing_fixture_decisions_under_both_authorities() {
    let previous_policy = crate::hot_field::process_policy();
    for policy in [
        crate::hot_field::HotFieldPolicy::ime(),
        crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
            crate::text_backend::TextBackendPreference::Uinput,
        ),
    ] {
        crate::hot_field::set_process_policy(policy);
        for fixture in EXISTING_ADMISSION_FIXTURES {
            let uncached = with_admission_fact_reuse(false, || {
                candidate_admission(
                    fixture.original,
                    fixture.replacement,
                    fixture.error_class,
                    fixture.origin,
                )
            });
            let reused = with_admission_fact_reuse(true, || {
                candidate_admission(
                    fixture.original,
                    fixture.replacement,
                    fixture.error_class,
                    fixture.origin,
                )
            });
            assert_eq!(
                reused, uncached,
                "policy={policy:?} original={:?} replacement={:?} class={:?} origin={:?}",
                fixture.original, fixture.replacement, fixture.error_class, fixture.origin
            );
        }
    }
    crate::hot_field::set_process_policy(previous_policy);
}

#[test]
fn lexical_fact_owner_is_lazy_call_local_and_uncached_mode_retains_nothing() {
    let unchanged = AdmissionLexicalFacts::with_mode("слово ", "слово ", true);
    let decision = candidate_admission_with_facts(
        "слово ",
        "слово ",
        TypingErrorClass::LetterSubstitution,
        CandidateOrigin::DeterministicTypo,
        &unchanged,
    );
    assert_eq!(decision.reason, "unchanged");
    assert_eq!(
        unchanged.snapshot(),
        AdmissionLexicalFactSnapshot::default()
    );

    let uncached = AdmissionLexicalFacts::with_mode("посмотри ", "посмотреть ", false);
    let _ = candidate_admission_with_facts(
        "посмотри ",
        "посмотреть ",
        TypingErrorClass::CompositeTypo,
        CandidateOrigin::L2Surface,
        &uncached,
    );
    assert_eq!(uncached.snapshot(), AdmissionLexicalFactSnapshot::default());

    let reused = AdmissionLexicalFacts::with_mode("посмотри ", "посмотреть ", true);
    let _ = candidate_admission_with_facts(
        "посмотри ",
        "посмотреть ",
        TypingErrorClass::CompositeTypo,
        CandidateOrigin::L2Surface,
        &reused,
    );
    let snapshot = reused.snapshot();
    assert!(snapshot.original_word && snapshot.replacement_word);
    assert!(snapshot.original_lower && snapshot.replacement_lower);
    assert!(snapshot.original_known && snapshot.replacement_known);
    assert!(snapshot.original_protected);
    assert!(!snapshot.replacement_protected);

    let next_call = AdmissionLexicalFacts::with_mode("дожь ", "дождь ", true);
    assert_eq!(
        next_call.snapshot(),
        AdmissionLexicalFactSnapshot::default()
    );
}

#[test]
fn short_cyrillic_to_ascii_layout_is_never_applyable_from_logs() {
    for (original, replacement) in [("40 000 р ", "40 000 h "), ("Екб ", "Tr, ")] {
        let gate = gate_candidate(original, replacement, TypingErrorClass::WrongLayout);

        assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
        assert_eq!(gate.reason, "short_cyrillic_to_ascii_layout");
    }
}

#[test]
fn exact_short_layout_projection_to_known_english_center_is_eligible() {
    let gate = gate_candidate_with_origin(
        "дфн ",
        "lay ",
        TypingErrorClass::WrongLayout,
        CandidateOrigin::Layout,
    );

    assert_eq!(gate.action, CandidateGateAction::Eligible, "{gate:?}");
}

#[test]
fn l2_cannot_delete_a_known_inflection_without_context_authority() {
    let gate = gate_candidate_with_origin(
        "в коде ",
        "в код ",
        TypingErrorClass::ExtraLetter,
        CandidateOrigin::L2Surface,
    );

    assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
    assert_eq!(gate.reason, "known_current_word_surface_drift");
}

#[test]
fn l2_cannot_rewrite_known_russian_surfaces_from_live_log() {
    for (original, replacement, error_class) in [
        ("закинем ", "закон ", TypingErrorClass::CompositeTypo),
        ("китайцы ", "китайы ", TypingErrorClass::ExtraLetter),
        ("ходу ", "ход ", TypingErrorClass::ExtraLetter),
        ("делаем ", "деваем ", TypingErrorClass::LetterSubstitution),
    ] {
        let gate = gate_candidate_with_origin(
            original,
            replacement,
            error_class,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(
            gate.action,
            CandidateGateAction::SuggestOnly,
            "{original:?} -> {replacement:?}: {gate:?}"
        );
        assert_ne!(
            gate.reason, "class_allows_apply",
            "{original:?} -> {replacement:?}: {gate:?}"
        );
    }
}

#[test]
fn boundary_shift_tail_pair_full_text_is_eligible() {
    for (original, replacement) in [
        ("допусти мнабираю ", "допустим набираю "),
        ("я думаю допусти мнабираю ", "я думаю допустим набираю "),
    ] {
        let gate = gate_candidate_with_origin(
            original,
            replacement,
            TypingErrorClass::BoundaryShift,
            CandidateOrigin::Boundary,
        );

        assert_eq!(gate.action, CandidateGateAction::Eligible, "{gate:?}");
    }
}
