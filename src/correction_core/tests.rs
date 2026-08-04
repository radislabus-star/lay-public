#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_typing_assist_pipeline;

    const SEMANTIC_WORD_FIXTURE_SOURCE: &str = "SemanticWordCell32";

    fn request<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
        mode: CorrectionMode,
    ) -> CorrectionRequest<'a> {
        CorrectionRequest {
            text,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            mode,
        }
    }

    #[test]
    fn l2_candidate_sources_follow_correction_mode_order() {
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::DeterministicOnly),
            &[L2CandidateSource::Deterministic]
        );
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::NandaOnly),
            &[L2CandidateSource::Nanda]
        );
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::DeterministicThenNanda),
            &[L2CandidateSource::Deterministic, L2CandidateSource::Nanda]
        );
    }

    #[test]
    fn delayed_context_births_previous_token_candidate_without_apply_authority() {
        let memory = crate::nanda_wave::llmwave::LlmWaveMemory::from_text(
            "всё ты сделал\nвсё ты понял\nвсё ты проверил\nвес ты измерил",
        );

        let candidates = delayed_context_candidates_with_memory("вес ты ", &memory);

        let candidate = candidates
            .iter()
            .find(|candidate| candidate.replacement == "всё ты ")
            .expect("reverse phrase candidate");
        assert_eq!(candidate.origin, CandidateOrigin::L3Context);
        assert_eq!(candidate.gate.action, CandidateGateAction::SuggestOnly);
    }

    #[test]
    fn l2_candidate_lattice_keeps_sources_and_selects_only_apply_candidate() {
        let mut lattice = L2CandidateLattice::new(TypingErrorEvent::from_text("автозаена "));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "автозамена ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "L2SurfaceMotifCell32",
            TypingErrorClass::MissingLetter,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        )));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "автозамена ",
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            ids::MISSING_LETTER,
            TypingErrorClass::MissingLetter,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        )));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "авто замена ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Boundary,
            "BoundaryCell32",
            TypingErrorClass::GluedWords,
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "requires_boundary_proof",
            },
        )));

        let resolution = lattice.into_resolution();

        assert_eq!(resolution.candidates.len(), 2);
        assert_eq!(
            resolution
                .candidates
                .iter()
                .filter(|candidate| candidate.replacement == "автозамена ")
                .count(),
            1,
            "duplicate same-replacement candidates must collapse into one evidence-backed node"
        );
        assert_eq!(resolution.scoreboard.total_candidates, 2);
        assert_eq!(resolution.scoreboard.deterministic_candidates, 1);
        assert_eq!(resolution.scoreboard.nanda_candidates, 2);
        assert_eq!(resolution.scoreboard.apply_candidates, 1);
        assert_eq!(resolution.scoreboard.suggest_only_candidates, 1);
        let selected = resolution.selected.as_ref().expect("selected candidate");
        assert_eq!(selected.replacement, "автозамена ");
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        assert_eq!(selected.evidence_count(), 2);
        assert!(selected.has_origin(CandidateOrigin::L2Surface));
        assert!(selected.has_origin(CandidateOrigin::DeterministicTypo));
        assert_eq!(
            resolution.decision,
            Some(CorrectionDecision {
                replacement: "автозамена ".to_string(),
                source: CorrectionDecisionSource::Nanda,
            })
        );
    }

    #[test]
    fn wave_owns_equal_verified_reconstruction_independent_of_source_order() {
        let mut lattice = L2CandidateLattice::new(TypingErrorEvent::from_text("охрошо "));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "хорошо ",
            CorrectionDecisionSource::Deterministic,
            CandidateOrigin::DeterministicTypo,
            ids::ADJACENT_TRANSPOSITION,
            TypingErrorClass::AdjacentTransposition,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        )));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "хорошо ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "L2SurfaceMotifCell32",
            TypingErrorClass::AdjacentTransposition,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "transition_core_authorized",
            },
        )));

        let resolution = lattice.into_resolution();
        let selected = resolution
            .selected
            .as_ref()
            .expect("selected reconstruction");
        assert_eq!(selected.origin, CandidateOrigin::L2Surface);
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert_eq!(selected.evidence_count(), 2);
        assert!(selected.has_origin(CandidateOrigin::DeterministicTypo));
    }

    #[test]
    fn verified_duplicate_evidence_is_not_lost_to_source_order() {
        let mut lattice = L2CandidateLattice::new(TypingErrorEvent::from_text("цусрфе "));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "wechat ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "L2WordAttractorCell32",
            TypingErrorClass::CompositeTypo,
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "unexplained_signal_loss",
            },
        )));
        lattice.push_source(Some(UnifiedCorrectionCandidate::new(
            "wechat ",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::Layout,
            "LayoutWordCell32",
            TypingErrorClass::WrongLayout,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "transition_core_authorized",
            },
        )));

        let resolution = lattice.into_resolution();
        let candidate = resolution.candidates.first().expect("merged candidate");
        assert_eq!(candidate.source_id, "LayoutWordCell32");
        assert_eq!(candidate.error_class, TypingErrorClass::WrongLayout);
        assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
        assert_eq!(candidate.evidence_count(), 2);
        assert!(resolution.selected.is_some());
    }

    #[test]
    fn exact_layout_projection_is_selected_without_missing_letter_recovery() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ltkfq ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("resolution={resolution:#?}"));
        assert_eq!(selected.replacement, "делай ");
        assert_eq!(selected.origin, CandidateOrigin::Layout);
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert_eq!(selected.source_id, "LayoutWordCell32");
        assert!(selected.has_origin(CandidateOrigin::LayoutThenTypo));
    }

    #[test]
    fn stable_layout_projection_precedes_secondary_typo_repair_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "cnjq ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("resolution={resolution:#?}"));
        assert_eq!(selected.replacement, "стой ");
        assert!(
            resolution
                .candidates
                .iter()
                .all(|candidate| candidate.replacement != "сотой "),
            "stable raw projection must not enter a second typo pass: {resolution:#?}"
        );
    }

    #[test]
    fn verified_duplicate_evidence_cannot_override_keep_or_veto() {
        for protected_action in [CandidateGateAction::KeepOriginal, CandidateGateAction::Veto] {
            let mut protected = UnifiedCorrectionCandidate::new(
                "wechat ",
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::Technical,
                "ProtectedSurfaceCell32",
                TypingErrorClass::ProtectedToken,
                CandidateGateDecision {
                    action: protected_action,
                    reason: "protected",
                },
            );
            protected.merge_evidence(UnifiedCorrectionCandidate::new(
                "wechat ",
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::Layout,
                "LayoutWordCell32",
                TypingErrorClass::WrongLayout,
                CandidateGateDecision {
                    action: CandidateGateAction::Eligible,
                    reason: "transition_core_authorized",
                },
            ));

            assert_eq!(protected.gate.action, protected_action);
            assert_eq!(protected.source_id, "ProtectedSurfaceCell32");
        }
    }

    #[test]
    fn l2_surface_candidate_cannot_apply_left_context_rewrite() {
        let gate = gate_candidate_with_origin(
            "коретка улитела ",
            "етка улитка ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_ne!(gate.reason, "class_allows_apply");
    }

    #[test]
    fn nanda_candidates_respect_request_wave_options() {
        let pipeline = default_typing_assist_pipeline();
        let active = resolve_text_correction(CorrectionRequest {
            text: "звгрузи ",
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: true,
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            mode: CorrectionMode::NandaOnly,
        });
        assert!(active.candidates.iter().any(|candidate| {
            candidate.has_source_id("L2WordAttractorCell32") && candidate.replacement == "загрузи "
        }));

        let disabled = resolve_text_correction(CorrectionRequest {
            text: "звгрузи ",
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: true,
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::with_disabled(&["L2SurfaceMotifCell32".to_string()]),
            mode: CorrectionMode::NandaOnly,
        });
        assert!(!disabled
            .candidates
            .iter()
            .any(|candidate| candidate.source_id == "L2SurfaceMotifCell32"));
    }

    #[test]
    fn live_canonical_l2_field_births_nanda_candidates_without_full_wave_authority() {
        use std::time::Instant;

        let pipeline = default_typing_assist_pipeline();
        let mut req = request("звгрузи ", &pipeline, CorrectionMode::NandaOnly);
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();

        let resolution = resolve_text_correction(req);
        let candidates = &resolution.candidates;

        assert!(!candidates.is_empty());
        assert!(candidates.iter().all(|candidate| {
            candidate.source == CorrectionDecisionSource::Nanda
                && candidate.origin == CandidateOrigin::L2Surface
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.source_id.starts_with("CanonicalL2Field")
                || candidate
                    .evidence
                    .iter()
                    .any(|evidence| evidence.source_id.starts_with("CanonicalL2Field"))
        }));
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source_id.starts_with("CanonicalL2Field")));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.replacement == "загрузи "));
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str()),
            Some("загрузи ")
        );

        let sample_count = std::env::var("LAY_CANONICAL_L2_FIELD_SAMPLES")
            .or_else(|_| std::env::var("LAY_L2_FIELD_SHADOW_SAMPLES"))
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(120)
            .max(1);
        let mut timings = Vec::with_capacity(sample_count);
        for _ in 0..sample_count {
            let mut req = request("звгрузи ", &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let started = Instant::now();
            let resolution = resolve_text_correction(req);
            timings.push(started.elapsed().as_micros() as u64);
            assert_eq!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str()),
                Some("загрузи ")
            );
        }
        timings.sort_unstable();
        let p50 = timings[timings.len() / 2];
        let p90 = timings[timings.len() * 90 / 100];
        let p99 = timings[timings.len() * 99 / 100];
        let max = *timings.last().expect("latency samples");
        eprintln!(
            "CanonicalL2Field correction route: n={} p50={}us p90={}us p99={}us max={}us",
            timings.len(),
            p50,
            p90,
            p99,
            max
        );
        if std::env::var_os("LAY_ENFORCE_CANONICAL_L2_FIELD_LATENCY_BUDGET").is_some()
            || std::env::var_os("LAY_ENFORCE_L2_FIELD_SHADOW_LATENCY_BUDGET").is_some()
        {
            assert!(p99 <= 5_000, "CanonicalL2Field p99 exceeded budget: {p99}us");
            assert!(max <= 10_000, "CanonicalL2Field max exceeded budget: {max}us");
        }
    }

    #[test]
    fn live_canonical_l2_field_applies_verified_two_content_boundary() {
        let pipeline = default_typing_assist_pipeline();
        let mut req = request(
            "Еленапросит ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        );
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();

        let resolution = resolve_text_correction(req);
        let selected = resolution
            .selected
            .as_ref()
            .expect("verified two-center boundary must remain in the live lattice");

        assert_eq!(selected.replacement, "Елена просит ");
        assert_eq!(selected.origin, CandidateOrigin::Boundary);
        assert_eq!(selected.error_class, TypingErrorClass::GluedWords);
        assert_eq!(selected.source_id, "CanonicalL2FieldBoundary");
    }

    #[test]
    fn live_l2_field_owner_blocks_reference_only_semantic_word_drift() {
        let pipeline = default_typing_assist_pipeline();
        for input in ["модель генерит ", "окончанием слов "] {
            let mut req = request(input, &pipeline, CorrectionMode::DeterministicThenNanda);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let resolution = resolve_text_correction(req);

            assert_eq!(
                resolution.decision, None,
                "live owner must preserve an already valid phrase: {resolution:#?}"
            );
        }
    }

    #[test]
    fn live_canonical_l2_field_applies_adjacent_transposition_center() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let resolutions: Vec<_> = [
            ("врмея ", "время "),
            ("поянл ", "понял "),
            ("понял сомтрю ", "понял смотрю "),
        ]
        .into_iter()
        .map(|(input, expected)| {
            let mut req = request(input, &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            (expected, resolve_text_correction(req))
        })
        .collect();
        crate::hot_field::set_process_policy(previous_policy);
        for (expected, resolution) in resolutions {
            let selected = resolution
                .selected
                .as_ref()
                .unwrap_or_else(|| panic!("L2 transposition center must apply: {resolution:#?}"));

            assert_eq!(selected.replacement, expected);
            assert_eq!(
                selected.error_class,
                TypingErrorClass::AdjacentTransposition
            );
            assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        }
    }

    #[test]
    fn operator_consensus_selects_transposition_over_generic_l2_drift() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let mut req = request(
            "понял сомтрю ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        );
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);
        crate::hot_field::set_process_policy(previous_policy);

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("operator consensus must resolve: {resolution:#?}"));
        assert_eq!(selected.replacement, "понял смотрю ");
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert!(selected.has_origin(CandidateOrigin::DeterministicTypo));
        assert!(selected.has_origin(CandidateOrigin::L2Surface));
    }

    #[test]
    fn operator_consensus_repairs_extra_letter_despite_generic_negative_memory() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let mut req = request(
            "преоверка ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        );
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);
        crate::hot_field::set_process_policy(previous_policy);

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("operator consensus must resolve: {resolution:#?}"));
        assert_eq!(selected.replacement, "проверка ");
        assert_eq!(selected.error_class, TypingErrorClass::ExtraLetter);
        assert!(selected.has_origin(CandidateOrigin::DeterministicTypo));
        assert!(selected.has_origin(CandidateOrigin::L2Surface));
    }

    #[test]
    fn phase_verified_l2_transposition_repairs_unlisted_word_form() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let mut req = request(
            "проевряю ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        );
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);
        crate::hot_field::set_process_policy(previous_policy);

        let selected = resolution.selected.as_ref().unwrap_or_else(|| {
            panic!("phase-verified transposition must resolve: {resolution:#?}")
        });
        assert_eq!(selected.replacement, "проверяю ");
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert!(selected.has_origin(CandidateOrigin::L2Surface));
    }

    #[test]
    fn phase_verified_transposition_competes_with_known_noisy_surface() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let mut req = request("ландо ", &pipeline, CorrectionMode::DeterministicThenNanda);
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);
        crate::hot_field::set_process_policy(previous_policy);

        let selected = resolution.selected.as_ref().unwrap_or_else(|| {
            panic!("L2 must compete with a known noisy surface: {resolution:#?}")
        });
        assert_eq!(selected.replacement, "ладно ");
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert!(selected.has_origin(CandidateOrigin::L2Surface));
    }

    #[test]
    fn unique_transposition_certificate_repairs_short_word() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let mut req = request("мжоет ", &pipeline, CorrectionMode::DeterministicThenNanda);
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);
        crate::hot_field::set_process_policy(previous_policy);

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("unique transposition must resolve: {resolution:#?}"));
        assert_eq!(selected.replacement, "может ");
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert!(selected.has_origin(CandidateOrigin::DeterministicTypo));
        assert!(selected.has_origin(CandidateOrigin::L2Surface));
    }

    #[test]
    fn deterministic_mode_corrects_wrong_layout_text() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected) in [
            ("lfdfq ", "давай "),
            ("rfr ", "как "),
            ("gthtdjhfxbdftncz ", "переворачивается "),
        ] {
            let decision = decide_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ))
            .unwrap_or_else(|| panic!("wrong-layout candidate missing for {input:?}"));
            assert_eq!(decision.replacement, expected, "input={input:?}");
            assert_eq!(decision.source, CorrectionDecisionSource::Deterministic);
        }
    }

    #[test]
    fn deterministic_mode_corrects_multiword_wrong_layout_tail() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "HF<JNF NTCN CFV ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("РАБОТА ТЕСТ САМ "),
            "resolution={resolution:#?}"
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.gate.action),
            Some(CandidateGateAction::Eligible)
        );
    }

    #[test]
    fn full_wave_selects_noisy_multiword_layout_as_one_verified_transition() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "djn nfrjt djn yt gthtdfhfxbdftncz ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("вот такое вот не переворачивается "),
            "resolution={resolution:#?}"
        );
        let selected = resolution.selected.as_ref().expect("selected transition");
        assert!(selected.has_source_id(ids::LAYOUT_EN_TO_RU));
        assert!(selected.has_source_id("LayoutSequenceCell32"));
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn deterministic_mode_corrects_multiword_wrong_layout_tail_with_context_pipeline() {
        let default_pipeline = default_typing_assist_pipeline();
        let pipeline = crate::typing_context::typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_pipeline,
            "HF<JNF NTCN CFV ",
        );
        let resolution = resolve_text_correction(CorrectionRequest {
            text: "HF<JNF NTCN CFV ",
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Normal,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            mode: CorrectionMode::DeterministicOnly,
        });

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("РАБОТА ТЕСТ САМ "),
            "resolution={resolution:#?}"
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.gate.action),
            Some(CandidateGateAction::Eligible)
        );
    }

    #[test]
    fn resolution_routes_missing_letter_through_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "автозаена ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "автозамена ");
        assert_eq!(selected.error_class, TypingErrorClass::MissingLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        assert!(resolution.scoreboard.total_candidates >= 1);
        assert_eq!(resolution.scoreboard.apply_candidates, 1);
        assert!(resolution.scoreboard.deterministic_candidates >= 1);
        assert_eq!(resolution.scoreboard.nanda_candidates, 0);
        assert!(
            resolution
                .scoreboard
                .selected_bayes_posterior_milli
                .is_some(),
            "selected candidate must expose Bayes posterior"
        );
        assert_eq!(
            resolution.candidate_scores.len(),
            resolution.scoreboard.total_candidates
        );
        let score = resolution
            .candidate_scores
            .iter()
            .find(|score| score.selected)
            .expect("selected score trace");
        assert_eq!(score.replacement, "автозамена ");
        assert_eq!(score.error_class, TypingErrorClass::MissingLetter);
        assert_eq!(score.action_operator, "restore_missing_letter");
        assert_eq!(score.action_proof, "typo");
        assert_eq!(score.gate_action, CandidateGateAction::Eligible);
        assert!(score.selected);
        assert!(score.likelihood_milli > 0);
        assert!(score.posterior_milli > 0);
        assert_eq!(score.l4_scene_action, "suggest");
        assert_eq!(score.l4_scene_reason, "resolved");
        assert!(score.l4_scene_milli > 0);
        assert!(score.decision_rank_milli > 0);
    }

    #[test]
    fn unexplained_signal_loss_blocks_l2_shortcut_candidate() {
        let gate = gate_candidate_with_source(
            "тоесть ",
            "есть ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "unexplained_signal_loss");
    }

    #[test]
    fn surface_candidate_cannot_apply_extra_left_context() {
        let gate = gate_candidate_with_source(
            "содержкой ",
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn surface_candidate_may_replace_only_current_word_with_same_prefix() {
        let gate = gate_candidate_with_source(
            "что получилось содержкой ",
            "что получилось содержать ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_ne!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn layout_candidate_cannot_add_context_to_single_word() {
        let gate = gate_candidate_with_source(
            "uрафике ",
            "на графике ",
            TypingErrorClass::WrongLayout,
            "LayoutWordCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn layout_candidate_may_rewrite_multiword_layout_tail() {
        let gate = gate_candidate_with_source(
            "HF<JNF NTCN CFV ",
            "РАБОТА ТЕСТ САМ ",
            TypingErrorClass::WrongLayout,
            ids::LAYOUT_EN_TO_RU,
        );

        assert_eq!(gate.action, CandidateGateAction::Eligible, "gate={gate:?}");
    }

    #[test]
    fn boundary_preserving_candidate_survives_explanation_gate() {
        let gate = gate_candidate_with_source(
            "тоесть ",
            "то есть ",
            TypingErrorClass::CompositeTypo,
            ids::PERSONAL_PHRASE,
        );

        assert_eq!(gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn boundary_shift_is_selected_by_the_unified_correction_core() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "я думаю допусти мнабираю ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("resolution={resolution:#?}"));
        assert_eq!(selected.replacement, "я думаю допустим набираю ");
        assert_eq!(selected.error_class, TypingErrorClass::BoundaryShift);
        assert_eq!(selected.origin, CandidateOrigin::Boundary);
    }

    #[test]
    fn clean_phrase_boundary_cannot_apply_a_shifted_alternative() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "я вижу видит фразу ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
    }

    #[test]
    fn boundary_shift_cannot_apply_on_clean_two_word_surfaces() {
        let pipeline = default_typing_assist_pipeline();
        for text in [
            "моему аакаунут ",
            "коле Азейбарджан ",
            "Переносимые операторы ",
        ] {
            let resolution = resolve_text_correction(request(
                text,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert!(
                resolution.selected.as_ref().is_none_or(|candidate| {
                    candidate.error_class != TypingErrorClass::BoundaryShift
                }),
                "text={text:?} resolution={resolution:#?}"
            );
        }
    }

    #[test]
    fn ambiguous_short_boundary_shift_is_suggestion_only() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "во тты ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "вот ты "
                && candidate.error_class == TypingErrorClass::BoundaryShift
        }));
    }

    #[test]
    fn ambiguous_long_l2_surface_drift_from_live_log_is_suggestion_only() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "самка схема парочинная ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        assert!(
            resolution
                .candidates
                .iter()
                .any(|candidate| candidate.replacement == "самка схема перочинная "),
            "resolution={resolution:#?}"
        );
    }

    #[test]
    fn split_phrase_candidate_wins_over_l2_shortcut() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "тоесть ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "то есть ");
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn unknown_russian_shape_is_classified_before_candidate_generation() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "приудишна ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert_eq!(resolution.event.current_word, "приудишна");
        assert_eq!(
            resolution.event.input_class,
            TypingErrorClass::CompositeTypo
        );
        assert_eq!(resolution.decision, None, "resolution={resolution:#?}");
    }

    #[test]
    fn l3_anti_shortcut_blocks_overcompressed_word_candidate() {
        let gate = gate_candidate_with_source(
            "патерна ",
            "пара ",
            TypingErrorClass::CompositeTypo,
            SEMANTIC_WORD_FIXTURE_SOURCE,
        );

        assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
        assert_eq!(gate.reason, "candidate_over_compresses_word");
    }

    #[test]
    fn l2_surface_cannot_apply_context_stem_truncation() {
        let gate = gate_candidate_with_source(
            "я прохоил ",
            "я проход ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "l2_surface_stem_truncation_low");
    }

    #[test]
    fn l3_anti_shortcut_blocks_function_prefix_letter_drop_from_logs() {
        let gate = gate_candidate_with_source(
            "ответили вчате ",
            "ответили вате ",
            TypingErrorClass::CompositeTypo,
            SEMANTIC_WORD_FIXTURE_SOURCE,
        );

        assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
        assert_eq!(gate.reason, "function_prefix_letter_drop");
    }

    #[test]
    fn l3_anti_shortcut_blocks_short_layout_without_phrase_context() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("wave b ", &pipeline, CorrectionMode::NandaOnly));

        assert_eq!(resolution.decision, None);
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.gate.action == CandidateGateAction::KeepOriginal
                    && candidate.gate.reason == "short_layout_without_phrase_context"
            }),
            "short layout candidate may be visible, but must stay powerless: {resolution:?}"
        );
    }

    #[test]
    fn known_russian_word_with_yo_is_not_layout_switched_to_ascii() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ещё ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.replacement != "to` "
                    && candidate.gate.action != CandidateGateAction::Eligible
            }),
            "known Russian word must not autoswitch to ASCII layout: {resolution:?}"
        );
    }

    #[test]
    fn short_russian_word_does_not_autoswitch_to_ascii_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        for (input, bad_replacement) in [("40 000 р ", "40 000 h "), ("Екб ", "Tr, ")] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}");
            let matching = resolution
                .candidates
                .iter()
                .filter(|candidate| candidate.replacement == bad_replacement)
                .collect::<Vec<_>>();
            assert!(
                matching.iter().all(|candidate| {
                    candidate.gate.action == CandidateGateAction::KeepOriginal
                        && candidate.gate.reason == "short_cyrillic_to_ascii_layout"
                }),
                "input={input:?} resolution={resolution:#?}"
            );
            if input == "Екб " {
                assert!(
                    !matching.is_empty(),
                    "three-letter log case must reach the structural gate: {resolution:#?}"
                );
            }
        }
    }

    #[test]
    fn russian_phrase_context_keeps_left_context_repair_suggest_only() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "читай cola d wechat ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none(), "{resolution:?}");
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "читай cola в wechat "
                && candidate.gate.action == CandidateGateAction::SuggestOnly
                && candidate.gate.reason == "edit_transition_not_verified"
        }));
    }

    #[test]
    fn layout_then_typo_repairs_dirty_wrong_layout_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "hf,jfntn ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "работает ");
        assert!(selected.has_origin(CandidateOrigin::Layout));
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn layout_then_known_word_does_not_flip_known_english_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "file ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.replacement != "ашду "
                    || candidate.gate.action != CandidateGateAction::Eligible
            }),
            "known English word must not use weak layout fallback: {resolution:?}"
        );
    }

    #[test]
    fn english_word_centers_beat_more_expensive_cross_script_projections() {
        let pipeline = default_typing_assist_pipeline();
        for (original, expected) in [("dowenload ", "download "), ("adress ", "address ")] {
            let resolution = resolve_text_correction(request(
                original,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));
            let decision = resolution
                .decision
                .unwrap_or_else(|| panic!("same-script candidate for {original:?}"));
            assert_eq!(decision.replacement, expected, "input={original:?}");
            assert_eq!(decision.source, CorrectionDecisionSource::Nanda);
        }
    }

    #[test]
    fn cyrillic_projection_can_settle_through_english_l2_word_center() {
        let pipeline = default_typing_assist_pipeline();
        for (original, expected) in [
            ("вщцутдщфв ", "download "),
            ("учьфзду ", "example "),
            ("фвкуыы ", "address "),
        ] {
            let mut req = request(original, &pipeline, CorrectionMode::DeterministicThenNanda);
            req.nanda_wave_options = req.nanda_wave_options.with_l2_phase_apply(true);
            let resolution = resolve_text_correction(req);
            let selected = resolution
                .selected
                .as_ref()
                .unwrap_or_else(|| panic!("layout+L2 candidate: {resolution:?}"));
            assert_eq!(selected.replacement, expected, "input={original:?}");
            assert_eq!(selected.origin, CandidateOrigin::LayoutThenTypo);
            assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        }
    }

    #[test]
    fn known_russian_centers_block_composite_layout_settling() {
        let pipeline = default_typing_assist_pipeline();
        for original in ["привет ", "проверка ", "работает ", "скачать "]
        {
            let mut req = request(original, &pipeline, CorrectionMode::DeterministicThenNanda);
            req.nanda_wave_options = req.nanda_wave_options.with_l2_phase_apply(true);
            let resolution = resolve_text_correction(req);
            assert!(
                resolution.selected.as_ref().map_or(true, |candidate| {
                    is_cyrillic_letters_only(candidate.replacement.trim())
                }),
                "known Russian center must retain script: {resolution:?}"
            );
        }
    }

    #[test]
    fn composite_typo_repairs_known_russian_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "помшник ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "помощник ");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn composite_typo_does_not_jump_over_known_single_step_repair() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "мы отвравим ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "мы отравим ");
        assert_ne!(selected.replacement, "мы отвратим ");
    }

    #[test]
    fn nanda_semantic_drift_does_not_beat_local_single_step_repair() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "мы отвравим ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.as_ref().expect("selected candidate");
        assert_eq!(selected.replacement, "мы отравим ", "{resolution:?}");
        assert_eq!(selected.source, CorrectionDecisionSource::Deterministic);
        assert!(
            resolution.candidates.iter().all(|candidate| {
                candidate.source != CorrectionDecisionSource::Nanda
                    || candidate.replacement != selected.replacement
            }),
            "NANDA must not steal deterministic ownership for the same replacement: {resolution:?}"
        );
        assert_ne!(selected.replacement, "мы отвратим ");
    }

    #[test]
    fn composite_gate_blocks_same_tail_consonant_semantic_drift() {
        let decision = gate_candidate_with_source(
            "будет примать ",
            "будет придать ",
            TypingErrorClass::CompositeTypo,
            SEMANTIC_WORD_FIXTURE_SOURCE,
        );

        assert_eq!(decision.action, CandidateGateAction::SuggestOnly);
        assert_eq!(decision.reason, "same_tail_single_consonant_drift");
    }

    #[test]
    fn l2_surface_single_letter_repair_from_dirty_surface_can_apply() {
        let decision = gate_candidate_with_origin(
            "видешь ",
            "видишь ",
            TypingErrorClass::LetterSubstitution,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(decision.action, CandidateGateAction::Eligible, "{decision:?}");
    }

    #[test]
    fn l2_surface_missing_letter_repair_from_dirty_surface_can_apply() {
        let decision = gate_candidate_with_origin(
            "дожь ",
            "дождь ",
            TypingErrorClass::MissingLetter,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(decision.action, CandidateGateAction::Eligible, "{decision:?}");
    }

    #[test]
    fn composite_typo_rejects_short_initial_consonant_growth() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "давай лушее ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.expect("safe candidate should remain");
        assert_eq!(selected.replacement, "давай лучшее ");
        assert!(!resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "давай глушее "
                && candidate.gate.action == CandidateGateAction::Eligible));
    }

    #[test]
    fn composite_typo_rejects_short_initial_vowel_growth_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [("рина ", "арина "), ("решение задачь ", "решение озадачь ")]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement)
                    != Some(&forbidden.to_string()),
                "forbidden candidate auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn known_russian_words_do_not_autorewrite_to_other_known_words() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("искать хрень! ", "искать хрену "),
            ("будет плох ", "будет плоха "),
            ("Блин ", "Блина "),
            ("не мение ", "не мерние "),
            ("не мение ", "не менте "),
            ("теорию бейса ", "теорию бейсяа "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement)
                    != Some(&forbidden.to_string()),
                "forbidden known-word rewrite auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn weak_shape_drift_from_live_logs_stays_suggest_only() {
        for (input, replacement, error_class, source_id) in [
            (
                "версии ",
                "версти ",
                TypingErrorClass::CompositeTypo,
                "composite_ru_typo",
            ),
            (
                "планы? ",
                "плауны? ",
                TypingErrorClass::CompositeTypo,
                "composite_ru_typo",
            ),
            (
                "нужен ",
                "ножен ",
                TypingErrorClass::LetterSubstitution,
                ids::VOWEL_CONFUSION,
            ),
            (
                "кодировании ",
                "кодированиеи ",
                TypingErrorClass::MissingLetter,
                ids::MISSING_LETTER,
            ),
            (
                "очереди ",
                "очередьи ",
                TypingErrorClass::CompositeTypo,
                "composite_ru_typo",
            ),
            (
                "пользоватся? ",
                "пользовается ",
                TypingErrorClass::CompositeTypo,
                SEMANTIC_WORD_FIXTURE_SOURCE,
            ),
        ] {
            let gate = gate_candidate_with_source(input, replacement, error_class, source_id);

            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "{input:?} -> {replacement:?}"
            );
            assert!(
                matches!(
                    gate.reason,
                    "known_current_word_surface_drift"
                        | "unproven_stable_surface_shape_drift"
                        | "known_word_to_different_known_word"
                ),
                "{input:?} -> {replacement:?}: {gate:?}"
            );
        }
    }

    #[test]
    fn weak_shape_drift_from_live_logs_is_not_selected() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("версии ", "версти "),
            ("планы? ", "плауны? "),
            ("нужен ", "ножен "),
            ("кодировании ", "кодирование "),
            ("кодировании ", "кодированиеи "),
            ("очереди ", "очередьи "),
            ("пользоватся? ", "пользовается "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str() != forbidden)
                    .unwrap_or(true),
                "{input:?} must not select {forbidden:?}; candidates={:?}",
                resolution.candidates
            );
        }
    }

    #[test]
    fn fresh_live_l2_false_applies_do_not_reach_autocorrect_decision() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("ая ", "яа "),
            ("ту ", "ут "),
            ("вно ", "вон "),
            ("ям ", "мя "),
            ("новости ", "новость "),
            ("модели ", "модель "),
            ("вышли ", "вышил "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_ne!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| decision.replacement.as_str()),
                Some(forbidden),
                "{input:?} must not auto-apply {forbidden:?}: {resolution:?}"
            );
        }
    }

    #[test]
    fn fresh_log_clean_russian_forms_are_preserved() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("могли ", "могил "),
            ("скажу ", "скажиу "),
            ("китайцев ", "китайев "),
            ("Пиши ", "Приши "),
            ("переделаем ", "переделам "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_ne!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| decision.replacement.as_str()),
                Some(forbidden),
                "clean form must not drift for {input:?}: {resolution:?}"
            );
        }
    }

    #[test]
    fn strong_local_typo_repairs_still_apply_after_shape_guard() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected) in [
            ("длеай ", "делай "),
            ("тарфик ", "трафик "),
            ("рабоатешь ", "работаешь "),
            ("агресивнее ", "агрессивнее "),
            ("дейстия ", "действия "),
            ("кнал ", "канал "),
            ("сбирать ", "собирать "),
            ("переспективнее ", "перспективнее "),
            ("отвликайся ", "отвлекайся "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            let selected = resolution
                .selected
                .as_ref()
                .unwrap_or_else(|| panic!("selected candidate for {input:?}: {resolution:?}"));
            assert_eq!(
                selected.replacement, expected,
                "input={input:?}; {resolution:?}"
            );
            assert_eq!(
                selected.gate.action,
                CandidateGateAction::Eligible,
                "input={input:?}"
            );
        }
    }

    #[test]
    fn composite_typo_recovers_common_word_with_broken_prefix() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "где эсперемнт ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "где эксперимент ");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn composite_typo_prefers_effective_over_affective_for_missing_initial_vowel() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько ффективная ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "на сколько эффективная ");
        assert_eq!(selected.error_class, TypingErrorClass::MissingLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn boundary_gate_does_not_split_known_single_word() {
        let gate = gate_candidate("уровне ", "у ровне ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "weak_boundary_split_tail");
    }

    #[test]
    fn boundary_gate_does_not_split_known_word_inside_phrase() {
        let gate = gate_candidate("на уровне ", "на у ровне ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "weak_boundary_split_tail");
    }

    #[test]
    fn boundary_gate_rejects_known_word_split_from_non_boundary_candidate() {
        let gate = gate_candidate_with_source(
            "за настройки ",
            "за нас тройки ",
            TypingErrorClass::CompositeTypo,
            SEMANTIC_WORD_FIXTURE_SOURCE,
        );

        assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
        assert_eq!(gate.reason, "candidate_over_compresses_word");
    }

    #[test]
    fn boundary_gate_rejects_short_function_split_with_unknown_tail() {
        let gate = gate_candidate("со скрина ", "со с крина ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn composite_typo_repairs_generated_russian_forms() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected, expected_class) in [
            ("руских ", "русских ", TypingErrorClass::MissingLetter),
            ("звгрузи ", "загрузи ", TypingErrorClass::LetterSubstitution),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            let selected = resolution
                .selected
                .as_ref()
                .unwrap_or_else(|| panic!("selected candidate for {input:?}: {resolution:?}"));
            assert_eq!(
                selected.replacement, expected,
                "input={input:?}; {resolution:?}"
            );
            assert_eq!(selected.error_class, expected_class, "input={input:?}");
            assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        }
    }

    #[test]
    fn known_phrase_parts_do_not_autogrow_by_one_letter() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("у меня ", "у меняю "),
            ("твой ", "тывой "),
            ("к тебе ", "к требе "),
            ("Тебе ", "Требе "),
            ("в план! ", "в плана! "),
            ("но пока ", "но прока "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}");
            assert!(
                resolution.candidates.iter().all(|candidate| {
                    candidate.replacement != forbidden
                        || candidate.gate.action != CandidateGateAction::Eligible
                }),
                "forbidden candidate auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn nanda_candidate_cannot_autogrow_known_phrase_part_either() {
        let gate = gate_candidate("твой ", "тывой ", TypingErrorClass::CompositeTypo);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_current_word_surface_drift");
    }

    #[test]
    fn reflexive_suffix_needs_grammar_proof_before_apply() {
        for source_id in ["composite_ru_typo", "L2SurfaceMotifCell32", "PhraseCell32"] {
            let gate = gate_candidate_with_source(
                "что нравится? ",
                "что нравиться? ",
                TypingErrorClass::MissingLetter,
                source_id,
            );

            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "source_id={source_id}"
            );
            assert_eq!(
                gate.reason, "reflexive_suffix_requires_grammar_proof",
                "source_id={source_id}"
            );
        }
    }

    #[test]
    fn grammar_source_may_handle_reflexive_suffix() {
        let gate = gate_candidate_with_source(
            "что нравится? ",
            "что нравиться? ",
            TypingErrorClass::GrammarAgreement,
            "GrammarCell32",
        );

        assert_ne!(gate.reason, "reflexive_suffix_requires_grammar_proof");
    }

    #[test]
    fn known_current_word_surface_drift_stays_suggest_only() {
        for (input, replacement, source_id) in [
            ("Читал логи ", "Читал логик ", "L2SurfaceMotifCell32"),
            ("смотри, ", "смотори, ", "composite_ru_typo"),
        ] {
            let gate = gate_candidate_with_source(
                input,
                replacement,
                TypingErrorClass::CompositeTypo,
                source_id,
            );

            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "{input:?} -> {replacement:?}"
            );
            assert_eq!(
                gate.reason, "known_current_word_surface_drift",
                "{input:?} -> {replacement:?}"
            );
        }
    }

    #[test]
    fn protected_multiword_tail_rewrite_stays_suggest_only() {
        for (replacement, error_class) in [
            ("давай там просмотри ", TypingErrorClass::MissingLetter),
            ("давай там подсмотри ", TypingErrorClass::MissingLetter),
            ("давай там досмотри ", TypingErrorClass::LetterSubstitution),
        ] {
            let gate = gate_candidate_with_source(
                "давай там посмотри ",
                replacement,
                error_class,
                "L2SurfaceMotifCell32",
            );

            assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
            assert_ne!(gate.reason, "class_allows_apply");
        }
    }

    #[test]
    fn known_finished_form_cannot_grow_into_infinitive_on_post_space_route() {
        let gate = gate_candidate_with_source(
            "посмотри ",
            "посмотреть ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_form_to_infinitive_overreach");
    }

    #[test]
    fn nanda_semantic_candidate_cannot_rewrite_known_word_to_neighbor_word() {
        let gate = gate_candidate(
            "искать хрень! ",
            "искать хрену ",
            TypingErrorClass::CompositeTypo,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_ne!(gate.reason, "class_allows_apply");
    }

    #[test]
    fn semantic_word_cell_far_surface_jumps_need_final_context_authority() {
        let pipeline = default_typing_assist_pipeline();
        for (input, replacement) in [
            ("реально помагаешь ", "реально понимаешь "),
            ("она спраивтя ", "она спрашивая "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));
            assert_ne!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| decision.replacement.as_str()),
                Some(replacement),
                "far semantic jump gained final authority: {resolution:#?}"
            );
        }
    }

    #[test]
    fn transition_core_cannot_override_live_protected_terms() {
        let pipeline = default_typing_assist_pipeline();
        for input in [
            "это патерн ",
            "в гугле ",
            "блять ",
            "слово грокать ",
            "тоже грокнулся. ",
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}: {resolution:?}");
            assert!(
                resolution.selected.is_none(),
                "input={input:?}: {resolution:?}"
            );
        }
    }

    #[test]
    fn nanda_surface_candidates_from_logs_are_suggest_only_when_surface_is_weak() {
        for (input, replacement, reason) in [
            ("тели ", "тел ", "short_nanda_word_shrink"),
            (
                "нас моного ",
                "нас мюоного ",
                "short_nanda_internal_vowel_growth",
            ),
        ] {
            let gate = gate_candidate_with_source(
                input,
                replacement,
                TypingErrorClass::CompositeTypo,
                "L2SurfaceMotifCell32",
            );

            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "input={input:?}"
            );
            assert_eq!(gate.reason, reason, "input={input:?}");
        }
    }

    #[test]
    fn composite_typo_splits_previous_glued_word_when_fixing_current_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ее простозальет свтеом ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "ее просто зальет светом ");
        assert_eq!(selected.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn composite_typo_does_not_glue_two_committed_words() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "реально ое ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(!resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "реальное "));
    }

    #[test]
    fn live_log_multi_word_drifts_do_not_autoreplace_neighbors() {
        let pipeline = default_typing_assist_pipeline();
        for input in ["мете ты ", "тут тоже ", "я позвол ", "мы токенов "]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_eq!(
                resolution.decision, None,
                "multi-word dirty log case must not auto-apply: {input:?}: {resolution:?}"
            );
        }
    }

    #[test]
    fn single_letter_boundary_beats_wrong_transposition_candidate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "посмотреть влогах ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution.selected.expect("selected boundary candidate");
        assert_eq!(selected.replacement, "посмотреть в логах ");
        assert_eq!(selected.source_id, "BoundaryCell32");
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        assert!(resolution.candidates.iter().all(|candidate| {
            candidate.replacement != "посмотреть волгах "
                || candidate.gate.action != CandidateGateAction::Eligible
        }));
    }

    #[test]
    fn repeated_letter_repairs_short_all_caps_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ТРУССС ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate, resolution={resolution:?}"));
        assert_eq!(selected.replacement, "ТРУС ");
        assert_eq!(selected.source_id, ids::REPEATED_LETTER);
        assert_eq!(selected.error_class, TypingErrorClass::RepeatedLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn close_verified_typo_transitions_abstain_without_context_signal() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ППОНИКАЕШЬ? ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None, "resolution={resolution:#?}");
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "ПОНИКАЕШЬ? "
                && candidate.gate.action == CandidateGateAction::Eligible
        }));
    }

    #[test]
    fn composite_typo_repairs_short_adjacent_transposition_in_phrase() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "имеет смылс ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "имеет смысл ");
        assert_eq!(selected.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn adjacent_transposition_keeps_already_known_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Ладно ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn adjacent_transposition_cannot_rewrite_l2_known_word_without_state_proof() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Мы с тобой ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
        assert!(!resolution.candidates.is_empty());
        assert!(resolution
            .candidate_scores
            .iter()
            .all(|candidate| !candidate.selected));
    }

    #[test]
    fn extra_letter_operator_cannot_damage_l2_backed_inflection() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "тысяч рублей ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
        assert!(resolution
            .candidate_scores
            .iter()
            .all(|candidate| !candidate.selected));
    }

    #[test]
    fn l2_field_cannot_delete_known_case_ending_without_context_proof() {
        let pipeline = default_typing_assist_pipeline();
        let mut req = request("в коде ", &pipeline, CorrectionMode::DeterministicThenNanda);
        req.nanda_candidate_route = CandidateReadoutRoute::CanonicalL2Field;
        let resolution = resolve_text_correction(req);

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        assert!(resolution.decision.is_none(), "resolution={resolution:#?}");
        assert!(resolution
            .candidate_scores
            .iter()
            .filter(|candidate| candidate.replacement == "в код ")
            .all(|candidate| !candidate.selected));
    }

    #[test]
    fn future_auxiliary_blocks_non_infinitive_typo_candidate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "будет несити ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn live_log_style_finished_form_is_not_auto_extended_to_infinitive() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "давай там посмотри ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert_ne!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("давай там посмотреть "),
            "{resolution:#?}"
        );
        assert!(
            resolution
                .candidates
                .iter()
                .filter(|candidate| candidate.replacement == "давай там посмотреть ")
                .all(|candidate| candidate.gate.action != CandidateGateAction::Eligible),
            "{resolution:#?}"
        );
    }

    #[test]
    fn live_canonical_l2_field_log_style_finished_form_is_not_auto_extended_to_infinitive() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let mut req = request(
            "давай там посмотри ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        );
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);
        crate::hot_field::set_process_policy(previous_policy);

        assert_ne!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("давай там посмотреть "),
            "{resolution:#?}"
        );
        let infinitive_candidates: Vec<_> = resolution
            .candidates
            .iter()
            .filter(|candidate| candidate.replacement == "давай там посмотреть ")
            .collect();
        assert!(
            infinitive_candidates
                .iter()
                .all(|candidate| candidate.gate.action != CandidateGateAction::Eligible),
            "{resolution:#?}"
        );
        if !infinitive_candidates.is_empty() {
            assert!(
                infinitive_candidates
                    .iter()
                    .all(|candidate| candidate.gate.reason == "known_form_to_infinitive_overreach"),
                "{resolution:#?}"
            );
        }
    }

    #[test]
    fn nanda_mode_corrects_wave_writer_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision =
            decide_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly))
                .expect("nanda should produce a layout candidate");
        assert_eq!(decision.replacement, "nanda ");
        assert_eq!(decision.source, CorrectionDecisionSource::Nanda);
    }

    #[test]
    fn nanda_candidate_also_passes_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "nanda ");
        assert_eq!(selected.error_class, TypingErrorClass::WrongLayout);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn nanda_surface_motif_can_apply_known_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("звгрузи ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "загрузи ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert!(selected.has_source_id("L2WordAttractorCell32"));
        assert_eq!(selected.error_class, TypingErrorClass::LetterSubstitution);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn nanda_word_form_center_can_apply_nonlocal_composite_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Азейбарджан ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected L2 word-form center: {resolution:#?}"));
        assert_eq!(selected.replacement, "Азербайджан ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert!(selected.has_source_id("L2WordAttractorCell32"));
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn nanda_word_form_center_can_apply_edge_and_internal_omission() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько ффетивная ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected L2 sparse omission center: {resolution:#?}"));
        assert_eq!(selected.replacement, "на сколько эффективная ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert!(selected.has_source_id("L2SurfaceMotifCell32"));
        assert_eq!(
            selected.error_class,
            TypingErrorClass::SparseInternalMultiOmission
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn default_route_applies_edge_and_internal_omission_center() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько ффетивная ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected default-route sparse omission center: {resolution:#?}"));
        assert_eq!(selected.replacement, "на сколько эффективная ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert!(selected.has_source_id("L2SurfaceMotifCell32"));
        assert_eq!(
            selected.error_class,
            TypingErrorClass::SparseInternalMultiOmission
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn sparse_internal_omission_beats_composite_suffix_drift() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько переподлчаю ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected sparse omission over suffix drift: {resolution:#?}"));
        assert_eq!(selected.replacement, "на сколько переподключаю ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert!(selected.has_source_id("L2WordAttractorCell32"));
        assert_eq!(
            selected.error_class,
            TypingErrorClass::SparseInternalMultiOmission
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn nanda_surface_drift_stays_out_of_autocorrect_apply() {
        let pipeline = default_typing_assist_pipeline();
        for (input, bad_replacement) in
            [("сысл ", "сыск "), ("дать ", "гать "), ("теком ", "телом ")]
        {
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));
            assert!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str() != bad_replacement)
                    .unwrap_or(true),
                "{input:?} must not select weak drift {bad_replacement:?}; candidates={:?}",
                resolution.candidates
            );
        }
    }

    #[test]
    fn incomplete_surface_is_not_autocorrected_after_space() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("делай пров ", &pipeline, CorrectionMode::NandaOnly));

        assert!(resolution.decision.is_none());
        assert!(resolution
            .candidates
            .iter()
            .all(|candidate| candidate.gate.action != CandidateGateAction::Eligible));
    }

    #[test]
    fn unlearned_domain_phrase_has_no_hardcoded_apply_authority() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none(), "{resolution:#?}");
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn nanda_does_not_correct_customs_actor_phrase_without_right_anchor() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Поставщик говорит что цена до склада нашего покупателя но таможен ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn disabled_runtime_flags_keep_original() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(CorrectionRequest {
            text: "lfdfq ",
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: false,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            mode: CorrectionMode::DeterministicThenNanda,
        });
        assert_eq!(decision, None);
    }
}
