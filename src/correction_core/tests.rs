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
            lexical_authority_frame: None,
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
    fn l2_candidate_sources_follow_correction_mode_selection() {
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::DeterministicOnly),
            &[L2CandidateSource::Deterministic]
        );
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::NandaOnly),
            &[L2CandidateSource::Nanda]
        );
        assert_eq!(
            L2CandidateSource::for_mode(CorrectionMode::DeterministicAndNanda),
            &[L2CandidateSource::Deterministic, L2CandidateSource::Nanda]
        );
        assert_eq!(live_correction_mode(false), CorrectionMode::DeterministicOnly);
        assert_eq!(
            live_correction_mode(true),
            CorrectionMode::DeterministicAndNanda
        );
    }

    #[test]
    fn td113_live_hybrid_experimental_repairs_fixed_cross_class_matrix() {
        let pipeline = default_typing_assist_pipeline();
        let mut failures = Vec::new();
        for (case_id, input, target, forbidden) in [
            ("surface_typo", "плозо ", "плохо ", None),
            ("single_character", "обьяснить ", "объяснить ", None),
            ("transposition", "верменно ", "временно ", None),
            ("split_boundary", "текст е ", "тексте ", None),
            ("shifted_boundary", "т ыпочитай ", "ты почитай ", None),
            (
                "missing_character_with_boundary_competitor",
                "протколах ",
                "протоколах ",
                Some("пр отколах "),
            ),
        ] {
            let mut hybrid = request(input, &pipeline, CorrectionMode::DeterministicAndNanda);
            hybrid.nanda_candidate_route = CandidateReadoutRoute::live_default();
            hybrid.correction_safety = CorrectionSafety::Experimental;

            let resolution = resolve_text_correction(hybrid);
            let selected = resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str());
            if selected != Some(target) {
                let candidates = resolution
                    .candidates
                    .iter()
                    .map(|candidate| {
                        format!(
                            "{:?}|{:?}|{}|{}|{}",
                            candidate.replacement,
                            candidate.gate.action,
                            candidate.gate.reason,
                            candidate.source_id,
                            candidate.error_class.as_str(),
                        )
                    })
                    .collect::<Vec<_>>();
                failures.push(format!(
                    "case_id={case_id} selected={selected:?} target={target:?} candidates={candidates:?}"
                ));
            }
            if let Some(forbidden) = forbidden {
                if selected == Some(forbidden) {
                    failures.push(format!(
                        "case_id={case_id} selected forbidden boundary competitor {forbidden:?}"
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn td113_hybrid_admits_each_repeated_deterministic_target_once() {
        let pipeline = default_typing_assist_pipeline();
        for (input, target, error_class, origin) in [
            (
                "плозо ",
                "плохо ",
                TypingErrorClass::LetterSubstitution,
                CandidateOrigin::DeterministicTypo,
            ),
            (
                "обьяснить ",
                "объяснить ",
                TypingErrorClass::LetterSubstitution,
                CandidateOrigin::DeterministicTypo,
            ),
            (
                "верменно ",
                "временно ",
                TypingErrorClass::AdjacentTransposition,
                CandidateOrigin::DeterministicTypo,
            ),
            (
                "т ыпочитай ",
                "ты почитай ",
                TypingErrorClass::BoundaryShift,
                CandidateOrigin::Boundary,
            ),
            (
                "протколах ",
                "протоколах ",
                TypingErrorClass::MissingLetter,
                CandidateOrigin::DeterministicTypo,
            ),
        ] {
            let mut hybrid = request(input, &pipeline, CorrectionMode::DeterministicAndNanda);
            hybrid.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let (resolution, admission_count) =
                crate::typing_transition::proposal_admission::with_admission_evaluation_count(
                    target,
                    error_class,
                    origin,
                    || resolve_text_correction(hybrid),
                );

            assert_eq!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str()),
                Some(target),
                "input={input:?}: {resolution:#?}"
            );
            assert_eq!(
                admission_count, 1,
                "input={input:?} target={target:?} must pass proposal admission exactly once"
            );
        }
    }

    #[test]
    fn td113_fullwave_same_surface_consensus_survives_surface_drift_guard() {
        let pipeline = default_typing_assist_pipeline();
        let mut hybrid = request(
            "плозо ",
            &pipeline,
            CorrectionMode::DeterministicAndNanda,
        );
        hybrid.nanda_candidate_route = CandidateReadoutRoute::FullWave;
        hybrid.correction_safety = CorrectionSafety::Experimental;

        let resolution = resolve_text_correction(hybrid);
        let target = resolution
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "плохо ")
            .expect("same-surface target must remain in the hybrid lattice");

        assert_eq!(target.gate.action, CandidateGateAction::Eligible);
        assert!(target.has_eligible_origin(CandidateOrigin::DeterministicTypo));
        assert!(target.has_eligible_origin(CandidateOrigin::L2Surface));
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str()),
            Some("плохо "),
            "a certified same-surface consensus must not be rejected only because the merged primary owner is L2Surface: {resolution:#?}"
        );

        let mut nanda_only = request("плозо ", &pipeline, CorrectionMode::NandaOnly);
        nanda_only.nanda_candidate_route = CandidateReadoutRoute::FullWave;
        nanda_only.correction_safety = CorrectionSafety::Experimental;
        assert!(
            resolve_text_correction(nanda_only).selected.is_none(),
            "an uncorroborated Nanda surface must not inherit hybrid authority"
        );
    }

    #[test]
    fn td113_hybrid_profiles_preserve_td112_apply_thresholds() {
        let pipeline = default_typing_assist_pipeline();
        for (case_id, input, target) in [
            ("experimental_typo", "плозо ", "плохо "),
            (
                "strict_hard_sign",
                "обьяснить ",
                "объяснить ",
            ),
            ("normal_boundary_merge", "текст е ", "тексте "),
        ] {
            for correction_safety in [
                CorrectionSafety::Strict,
                CorrectionSafety::Normal,
                CorrectionSafety::Experimental,
            ] {
                let mut deterministic =
                    request(input, &pipeline, CorrectionMode::DeterministicOnly);
                deterministic.nanda_candidate_route = CandidateReadoutRoute::live_default();
                deterministic.correction_safety = correction_safety;
                let deterministic = resolve_text_correction(deterministic);

                let mut hybrid =
                    request(input, &pipeline, CorrectionMode::DeterministicAndNanda);
                hybrid.nanda_candidate_route = CandidateReadoutRoute::live_default();
                hybrid.correction_safety = correction_safety;
                let resolution = resolve_text_correction(hybrid);
                let retained = resolution
                    .candidates
                    .iter()
                    .find(|candidate| candidate.replacement == target)
                    .unwrap_or_else(|| {
                        panic!(
                            "case_id={case_id} profile={correction_safety:?}: {resolution:#?}"
                        )
                    });

                assert_eq!(retained.gate.action, CandidateGateAction::Eligible);
                assert_eq!(
                    resolution
                        .selected
                        .as_ref()
                        .map(|candidate| candidate.replacement.as_str()),
                    deterministic
                        .selected
                        .as_ref()
                        .map(|candidate| candidate.replacement.as_str()),
                    "case_id={case_id} profile={correction_safety:?}: deterministic={deterministic:#?} hybrid={resolution:#?}"
                );
                assert_eq!(
                    resolution.selected_transition.is_some(),
                    deterministic.selected_transition.is_some(),
                    "case_id={case_id} profile={correction_safety:?}: deterministic={deterministic:#?} hybrid={resolution:#?}"
                );
            }
        }
    }

    #[test]
    fn td113_hybrid_preserves_verified_mixed_layout_apply() {
        let pipeline = default_typing_assist_pipeline();
        for correction_safety in [
            CorrectionSafety::Strict,
            CorrectionSafety::Normal,
            CorrectionSafety::Experimental,
        ] {
            let mut hybrid = request(
                "fвтозамена ",
                &pipeline,
                CorrectionMode::DeterministicAndNanda,
            );
            hybrid.nanda_candidate_route = CandidateReadoutRoute::live_default();
            hybrid.correction_safety = correction_safety;
            let resolution = resolve_text_correction(hybrid);

            assert_eq!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str()),
                Some("автозамена "),
                "profile={correction_safety:?}: {resolution:#?}"
            );
            let transition = resolution
                .candidate_scores
                .iter()
                .find(|candidate| candidate.selected)
                .expect("mixed-layout Apply must carry a selected score trace");
            assert_eq!(
                transition.edit_transition_operator_kind,
                crate::text_edit::TransitionOperator::LayoutProjection
            );
            assert!(transition.edit_transition_verified);
        }
    }

    #[test]
    fn td113_hybrid_negative_matrix_has_zero_false_accepts() {
        let pipeline = default_typing_assist_pipeline();
        let mut observations = 0usize;
        let mut false_accepts = 0usize;
        for (case_id, input) in [
            ("clean_known_form", "новости "),
            ("ascii_technical", "cargo "),
            ("versioned_technical", "rustc-1.97.1 "),
            ("url", "https://example.org "),
            ("punctuation", "... "),
            ("short_ambiguous", "пку "),
            ("completed_form", "давай там посмотри "),
            ("live_protected", "блять "),
            ("russian_technical", "грокать "),
        ] {
            for correction_safety in [
                CorrectionSafety::Strict,
                CorrectionSafety::Normal,
                CorrectionSafety::Experimental,
            ] {
                let mut hybrid =
                    request(input, &pipeline, CorrectionMode::DeterministicAndNanda);
                hybrid.nanda_candidate_route = CandidateReadoutRoute::live_default();
                hybrid.correction_safety = correction_safety;
                let resolution = resolve_text_correction(hybrid);
                observations += 1;
                false_accepts += usize::from(resolution.selected.is_some());

                assert!(
                    resolution.selected.is_none(),
                    "case_id={case_id} profile={correction_safety:?}: {resolution:#?}"
                );
                assert!(resolution.selected_transition.is_none());
            }
        }

        assert_eq!(observations, 27);
        assert_eq!(false_accepts, 0);
    }

    #[test]
    #[ignore = "TD-113 paired performance proof; run explicitly with at least 60 samples per mode"]
    fn td113_hybrid_paired_performance_budget() {
        use std::time::Instant;

        fn process_cpu_us() -> u64 {
            let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
            let status = unsafe {
                libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, value.as_mut_ptr())
            };
            assert_eq!(status, 0, "CLOCK_PROCESS_CPUTIME_ID must be available");
            let value = unsafe { value.assume_init() };
            (value.tv_sec as u64)
                .saturating_mul(1_000_000)
                .saturating_add((value.tv_nsec as u64) / 1_000)
        }

        fn resident_rss_kib() -> u64 {
            std::fs::read_to_string("/proc/self/status")
                .expect("read /proc/self/status")
                .lines()
                .find_map(|line| line.strip_prefix("VmRSS:"))
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse::<u64>().ok())
                .expect("VmRSS in /proc/self/status")
        }

        fn percentile(values: &mut [u64], percentile: usize) -> u64 {
            values.sort_unstable();
            values[(values.len() - 1) * percentile / 100]
        }

        let sample_count = std::env::var("LAY_TD113_PERFORMANCE_SAMPLES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(60)
            .max(60);
        let inputs = [
            "плозо ",
            "обьяснить ",
            "верменно ",
            "текст е ",
            "т ыпочитай ",
            "протколах ",
        ];
        let pipeline = default_typing_assist_pipeline();

        for mode in [
            CorrectionMode::NandaOnly,
            CorrectionMode::DeterministicAndNanda,
        ] {
            for input in inputs {
                let mut req = request(input, &pipeline, mode);
                req.nanda_candidate_route = CandidateReadoutRoute::live_default();
                let _ = resolve_text_correction(req);
            }
        }

        let mut nanda_wall_us = Vec::with_capacity(sample_count);
        let mut hybrid_wall_us = Vec::with_capacity(sample_count);
        let mut nanda_cpu_us = Vec::with_capacity(sample_count);
        let mut hybrid_cpu_us = Vec::with_capacity(sample_count);
        for index in 0..sample_count {
            let input = inputs[index % inputs.len()];
            let order = if index % 2 == 0 {
                [
                    CorrectionMode::NandaOnly,
                    CorrectionMode::DeterministicAndNanda,
                ]
            } else {
                [
                    CorrectionMode::DeterministicAndNanda,
                    CorrectionMode::NandaOnly,
                ]
            };
            for mode in order {
                let mut req = request(input, &pipeline, mode);
                req.nanda_candidate_route = CandidateReadoutRoute::live_default();
                let cpu_started = process_cpu_us();
                let wall_started = Instant::now();
                let _ = resolve_text_correction(req);
                let wall_us = wall_started.elapsed().as_micros() as u64;
                let cpu_us = process_cpu_us().saturating_sub(cpu_started);
                match mode {
                    CorrectionMode::NandaOnly => {
                        nanda_wall_us.push(wall_us);
                        nanda_cpu_us.push(cpu_us);
                    }
                    CorrectionMode::DeterministicAndNanda => {
                        hybrid_wall_us.push(wall_us);
                        hybrid_cpu_us.push(cpu_us);
                    }
                    CorrectionMode::DeterministicOnly => unreachable!("paired modes are fixed"),
                }
            }
        }

        let rss_before_hybrid_kib = resident_rss_kib();
        for index in 0..sample_count {
            let input = inputs[index % inputs.len()];
            let mut req = request(input, &pipeline, CorrectionMode::DeterministicAndNanda);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let _ = resolve_text_correction(req);
        }
        let rss_after_hybrid_kib = resident_rss_kib();
        let rss_delta_kib = rss_after_hybrid_kib.saturating_sub(rss_before_hybrid_kib);

        let nanda_p50_us = percentile(&mut nanda_wall_us.clone(), 50);
        let nanda_p99_us = percentile(&mut nanda_wall_us, 99);
        let hybrid_p50_us = percentile(&mut hybrid_wall_us.clone(), 50);
        let hybrid_p99_us = percentile(&mut hybrid_wall_us, 99);
        let nanda_mean_cpu_us = nanda_cpu_us.iter().sum::<u64>() / sample_count as u64;
        let hybrid_mean_cpu_us = hybrid_cpu_us.iter().sum::<u64>() / sample_count as u64;

        eprintln!(
            "TD113_PAIRED_PERFORMANCE samples_per_mode={sample_count} nanda_p50_us={nanda_p50_us} hybrid_p50_us={hybrid_p50_us} nanda_p99_us={nanda_p99_us} hybrid_p99_us={hybrid_p99_us} nanda_mean_cpu_us={nanda_mean_cpu_us} hybrid_mean_cpu_us={hybrid_mean_cpu_us} rss_before_hybrid_kib={rss_before_hybrid_kib} rss_after_hybrid_kib={rss_after_hybrid_kib} rss_delta_kib={rss_delta_kib}"
        );
        assert!(
            hybrid_p50_us <= nanda_p50_us.saturating_add(5_000),
            "hybrid p50 regression: {hybrid_p50_us}us vs {nanda_p50_us}us"
        );
        assert!(
            hybrid_p99_us <= nanda_p99_us.saturating_add(10_000),
            "hybrid p99 regression: {hybrid_p99_us}us vs {nanda_p99_us}us"
        );
        assert!(
            hybrid_mean_cpu_us <= nanda_mean_cpu_us.saturating_add(10_000),
            "hybrid CPU regression: {hybrid_mean_cpu_us}us vs {nanda_mean_cpu_us}us"
        );
        assert!(
            rss_delta_kib <= 16 * 1024,
            "hybrid retained RSS regression: {rss_delta_kib} KiB"
        );
    }

    #[test]
    fn td112_profile_matrix_differentiates_fullwave_apply() {
        let pipeline = default_typing_assist_pipeline();
        let resolve = |correction_safety| {
            let mut request = request("звгрузи ", &pipeline, CorrectionMode::NandaOnly);
            request.correction_safety = correction_safety;
            resolve_text_correction(request)
        };

        let strict = resolve(CorrectionSafety::Strict);
        let normal = resolve(CorrectionSafety::Normal);
        let experimental = resolve(CorrectionSafety::Experimental);
        let target = "загрузи ";
        let target_metadata = |resolution: &CorrectionResolution| {
            let candidate = resolution
                .candidates
                .iter()
                .find(|candidate| candidate.replacement == target)
                .expect("fixed fallback target must remain in the lattice");
            (
                candidate.source,
                candidate.origin,
                candidate.source_id.clone(),
                candidate.error_class,
                candidate.gate.clone(),
                candidate.evidence_count(),
            )
        };

        assert_eq!(target_metadata(&strict), target_metadata(&experimental));
        assert_eq!(target_metadata(&normal), target_metadata(&experimental));
        assert_eq!(
            experimental
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str()),
            Some(target),
            "Experimental is the accepted 1.0.60 compatibility baseline"
        );
        assert_eq!(
            normal
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str()),
            Some(target),
            "Normal admits a FullWave candidate with one independent evidence domain"
        );
        assert!(
            strict.selected.is_none(),
            "Strict must retain the candidate but deny a one-domain FullWave Apply: {:#?}",
            strict.selected
        );
        assert!(strict.selected_transition.is_none());
    }

    #[test]
    fn td112_registered_and_fallback_deterministic_routes_match_r01_r02() {
        let pipeline = default_typing_assist_pipeline();
        for (case_id, text, target, expected_apply) in [
            ("R01", "автозаена ", "автозамена ", [false, true, true]),
            ("R02", "плозо ", "плохо ", [false, false, true]),
        ] {
            for (correction_safety, should_apply) in [
                CorrectionSafety::Strict,
                CorrectionSafety::Normal,
                CorrectionSafety::Experimental,
            ]
            .into_iter()
            .zip(expected_apply)
            {
                let mut req = request(text, &pipeline, CorrectionMode::DeterministicOnly);
                req.correction_safety = correction_safety;
                let resolution = resolve_text_correction(req);
                let retained = resolution
                    .candidates
                    .iter()
                    .find(|candidate| candidate.replacement == target)
                    .unwrap_or_else(|| {
                        panic!(
                            "case_id={case_id} profile={correction_safety:?}: {resolution:#?}"
                        )
                    });
                assert_eq!(retained.source, CorrectionDecisionSource::Deterministic);
                assert_eq!(retained.origin, CandidateOrigin::DeterministicTypo);
                assert_eq!(retained.gate.action, CandidateGateAction::Eligible);
                assert_eq!(
                    resolution
                        .selected
                        .as_ref()
                        .map(|candidate| candidate.replacement.as_str()),
                    should_apply.then_some(target),
                    "case_id={case_id} profile={correction_safety:?} retained={retained:#?}"
                );
                assert_eq!(resolution.selected_transition.is_some(), should_apply);
            }
        }
    }

    #[test]
    fn td112_merged_deterministic_and_nanda_aliases_match_r05() {
        let mut observed_metadata = None;
        for correction_safety in [
            CorrectionSafety::Strict,
            CorrectionSafety::Normal,
            CorrectionSafety::Experimental,
        ] {
            let mut lattice = L2CandidateLattice::with_options(
                TypingErrorEvent::from_text("автозаена "),
                &WaveOptions::default(),
                correction_safety,
            );
            lattice.push_source(Some(UnifiedCorrectionCandidate::new(
                "автозамена ",
                CorrectionDecisionSource::Nanda,
                CandidateOrigin::L2Surface,
                "L2SurfaceMotifCell32",
                TypingErrorClass::MissingLetter,
                CandidateGateDecision {
                    action: CandidateGateAction::Eligible,
                    reason: "td112_r05_nanda",
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
                    reason: "td112_r05_deterministic",
                },
            )));

            let resolution = lattice.into_resolution();
            let retained = resolution
                .candidates
                .iter()
                .find(|candidate| candidate.replacement == "автозамена ")
                .expect("R05 merged candidate remains in the lattice");
            let metadata = (
                retained.source,
                retained.origin,
                retained.error_class,
                retained.gate.action,
                retained.evidence_count(),
            );
            if let Some(expected) = observed_metadata {
                assert_eq!(metadata, expected, "profile={correction_safety:?}");
            } else {
                observed_metadata = Some(metadata);
            }
            assert_eq!(retained.evidence_count(), 2);
            assert_eq!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str()),
                Some("автозамена "),
                "R05 profile={correction_safety:?}: {resolution:#?}"
            );
            assert!(resolution.selected_transition.is_some());
        }
    }

    #[test]
    fn td112_canonical_and_high_precision_boundary_profiles_match_corpus() {
        let pipeline = default_typing_assist_pipeline();
        for (text, target, route, expected_apply) in [
            (
                "данорм ",
                "да норм ",
                CandidateReadoutRoute::CanonicalL2Field,
                [false, true, true],
            ),
            (
                "я думаю допусти мнабираю ",
                "я думаю допустим набираю ",
                CandidateReadoutRoute::FullWave,
                [true, true, true],
            ),
        ] {
            for (correction_safety, should_apply) in [
                CorrectionSafety::Strict,
                CorrectionSafety::Normal,
                CorrectionSafety::Experimental,
            ]
            .into_iter()
            .zip(expected_apply)
            {
                let mut req = request(text, &pipeline, CorrectionMode::NandaOnly);
                req.correction_safety = correction_safety;
                req.nanda_candidate_route = route;
                let resolution = resolve_text_correction(req);
                let retained = resolution
                    .candidates
                    .iter()
                    .find(|candidate| candidate.replacement == target)
                    .unwrap_or_else(|| {
                        panic!(
                            "profile={correction_safety:?} route={route:?}: {resolution:#?}"
                        )
                    });
                assert_eq!(
                    resolution
                        .selected
                        .as_ref()
                        .map(|candidate| candidate.replacement.as_str()),
                    should_apply.then_some(target),
                    "profile={correction_safety:?} route={route:?} retained={retained:#?}"
                );
                assert_eq!(resolution.selected_transition.is_some(), should_apply);
            }
        }
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
        let resolution =
            resolve_text_correction(request("ltkfq ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("resolution={resolution:#?}"));
        assert_eq!(selected.replacement, "делай ");
        assert_eq!(selected.origin, CandidateOrigin::Layout);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        assert!(
            !selected.has_origin(CandidateOrigin::LayoutThenTypo),
            "an exact layout projection must not inherit secondary typo authority: {selected:?}"
        );
    }

    #[test]
    fn stable_layout_projection_precedes_secondary_typo_repair_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("cnjq ", &pipeline, CorrectionMode::NandaOnly));

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
        assert_eq!(selected.origin, CandidateOrigin::Layout);
        assert!(!selected.has_origin(CandidateOrigin::LayoutThenTypo));
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
            lexical_authority_frame: None,
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
            lexical_authority_frame: None,
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
    #[ignore = "requires pinned L1.1, canonical L2, and Productive V90 packages; covered by immutable package integration proof"]
    fn td007_pinned_canonical_route_reconciles_historical_cases() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );
        let pipeline = default_typing_assist_pipeline();
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/td007-package-set-v1.json"
        ))
        .expect("TD-007 package manifest");
        let cases = manifest["historical_cases"]
            .as_array()
            .expect("TD-007 historical package cases");
        assert_eq!(cases.len(), 8);
        let mut evaluated_cases = 0;
        let mut nonempty_cases = 0;

        for case in cases {
            let input = case["input"].as_str().expect("case input");
            let expected = case["expected"].as_str().expect("case target");
            let expected_status = case["expected_status"].as_str().expect("case status");
            if expected_status == "COVERED_BY_L11_PRODUCER" {
                continue;
            }
            assert_eq!(expected_status, "ABSTAIN_NO_UNVERIFIED_APPLY");
            let mut req = request(input, &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let resolution = resolve_text_correction(req);
            evaluated_cases += 1;
            nonempty_cases += usize::from(!resolution.candidates.is_empty());

            assert!(resolution.selected.is_none(), "input={input:?}");
            assert!(resolution.decision.is_none(), "input={input:?}");
            assert!(resolution.selected_transition.is_none(), "input={input:?}");
            assert!(resolution.candidates.iter().all(|candidate| {
                candidate.source == CorrectionDecisionSource::Nanda
                    && candidate.has_origin(CandidateOrigin::L2Surface)
                    && candidate.source_id.starts_with("ProductiveL2V90")
                    && candidate.gate.action == CandidateGateAction::SuggestOnly
            }));
            if let Some(target) = resolution
                .candidates
                .iter()
                .find(|candidate| candidate.replacement == expected)
            {
                assert_eq!(target.source, CorrectionDecisionSource::Nanda);
                assert!(target.has_origin(CandidateOrigin::L2Surface));
                assert!(target.source_id.starts_with("ProductiveL2V90"));
                assert_eq!(target.gate.action, CandidateGateAction::SuggestOnly);
            }
        }
        assert_eq!(evaluated_cases, 7);
        assert!(nonempty_cases > 0, "pinned Productive route was never exercised");
        crate::hot_field::set_process_policy(previous_policy);
    }

    #[test]
    fn live_canonical_l2_field_stays_under_latency_budget() {
        use std::time::Instant;

        let pipeline = default_typing_assist_pipeline();
        let mut warmup = request("звгрузи ", &pipeline, CorrectionMode::NandaOnly);
        warmup.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let _warmup_resolution = resolve_text_correction(warmup);
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
            let _resolution = resolve_text_correction(req);
            timings.push(started.elapsed().as_micros() as u64);
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
            assert!(
                p99 <= 5_000,
                "CanonicalL2Field p99 exceeded budget: {p99}us"
            );
            assert!(
                max <= 10_000,
                "CanonicalL2Field max exceeded budget: {max}us"
            );
        }
    }

    #[test]
    #[ignore = "requires the canonical packages, L1.1 service, and a fixed surface corpus"]
    fn live_canonical_l2_field_reports_diverse_first_touch_latency() {
        use std::collections::BTreeSet;
        use std::time::Instant;

        let input_path = std::env::var("LAY_CANONICAL_L2_FIRST_TOUCH_INPUTS")
            .expect("LAY_CANONICAL_L2_FIRST_TOUCH_INPUTS must name the fixed surface corpus");
        let sample_count = std::env::var("LAY_CANONICAL_L2_FIRST_TOUCH_SAMPLES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(50)
            .max(1);
        let inputs = std::fs::read_to_string(&input_path)
            .expect("fixed first-touch surface corpus must be readable")
            .lines()
            .map(str::trim)
            .filter(|surface| !surface.is_empty())
            .map(|surface| format!("{surface} "))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        assert!(
            inputs.len() > sample_count,
            "first-touch corpus needs at least {} unique surfaces, found {}",
            sample_count + 1,
            inputs.len()
        );

        let pipeline = default_typing_assist_pipeline();
        let mut warmup = request(&inputs[0], &pipeline, CorrectionMode::NandaOnly);
        warmup.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let _ = resolve_text_correction(warmup);

        let mut timings = Vec::with_capacity(sample_count);
        for input in inputs.iter().skip(1).take(sample_count) {
            let mut req = request(input, &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let started = Instant::now();
            let _ = resolve_text_correction(req);
            timings.push(started.elapsed().as_micros() as u64);
        }
        timings.sort_unstable();
        let p50 = timings[timings.len() / 2];
        let p90 = timings[timings.len() * 90 / 100];
        let p99 = timings[timings.len() * 99 / 100];
        let max = *timings.last().expect("first-touch latency samples");
        eprintln!(
            "CanonicalL2Field diverse first touch: n={} p50={}us p90={}us p99={}us max={}us",
            timings.len(),
            p50,
            p90,
            p99,
            max
        );
    }

    #[test]
    fn live_canonical_l2_field_applies_verified_two_content_boundary() {
        let pipeline = default_typing_assist_pipeline();
        let mut req = request("Еленапросит ", &pipeline, CorrectionMode::NandaOnly);
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
    fn live_canonical_l2_field_applies_verified_short_left_boundary() {
        let pipeline = default_typing_assist_pipeline();
        let mut req = request("данорм ", &pipeline, CorrectionMode::NandaOnly);
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();

        let resolution = resolve_text_correction(req);
        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("short-left field boundary lost: {resolution:#?}"));

        assert_eq!(selected.replacement, "да норм ");
        assert_eq!(selected.origin, CandidateOrigin::Boundary);
        assert_eq!(selected.error_class, TypingErrorClass::GluedWords);
        assert_eq!(selected.source_id, "CanonicalL2FieldBoundary");
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("да норм ")
        );
        let transition = resolution
            .selected_transition
            .as_ref()
            .expect("DecisionCore boundary receipt")
            .diagnostic_transition();
        assert!(transition.is_verified());
        assert_eq!(
            transition.operator(),
            Some(crate::text_edit::TransitionOperator::BoundaryMergeSplit)
        );
        assert_eq!(
            transition.proof(),
            Some(crate::text_edit::TransitionProof::Boundary)
        );
    }

    #[test]
    fn live_canonical_l2_field_retains_split_evidence_without_applying_over_close_repair() {
        let pipeline = default_typing_assist_pipeline();
        for correction_safety in [CorrectionSafety::Normal, CorrectionSafety::Experimental] {
            let mut req = request("авторручка ", &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            req.correction_safety = correction_safety;

            let resolution = resolve_text_correction(req);
            let boundary = resolution
                .candidates
                .iter()
                .find(|candidate| candidate.replacement == "автор ручка ")
                .expect("producer-routed two-content boundary candidate");

            assert!(
                boundary.has_l2_boundary_target_grounding(),
                "DecisionCore competition must not erase target-bound L2 evidence"
            );
            assert_ne!(
                resolution
                    .selected
                    .as_ref()
                    .map(|candidate| candidate.replacement.as_str()),
                Some("автор ручка "),
                "a plausible one-word repair must keep a two-content split from auto-apply under {correction_safety:?}: {resolution:#?}"
            );
        }
    }

    #[test]
    fn td113_live_modes_do_not_apply_ambiguous_function_word_split() {
        let pipeline = default_typing_assist_pipeline();
        for mode in [
            CorrectionMode::NandaOnly,
            CorrectionMode::DeterministicAndNanda,
        ] {
            for correction_safety in [CorrectionSafety::Normal, CorrectionSafety::Experimental] {
                let mut req = request("воротаим ", &pipeline, mode);
                req.nanda_candidate_route = CandidateReadoutRoute::live_default();
                req.correction_safety = correction_safety;

                let resolution = resolve_text_correction(req);
                assert_ne!(
                    resolution
                        .selected
                        .as_ref()
                        .map(|candidate| candidate.replacement.as_str()),
                    Some("ворота им "),
                    "surface-only ambiguity must remain non-authoritative for {mode:?}/{correction_safety:?}: {resolution:#?}"
                );
            }
        }
    }

    #[test]
    fn td113_live_modes_preserve_grounded_boundary_positives() {
        let pipeline = default_typing_assist_pipeline();
        for mode in [
            CorrectionMode::NandaOnly,
            CorrectionMode::DeterministicAndNanda,
        ] {
            for correction_safety in [CorrectionSafety::Normal, CorrectionSafety::Experimental] {
                for (input, expected) in [
                    ("Какие документыим ", "Какие документы им "),
                    ("Готовь документыдля ", "Готовь документы для "),
                    ("Еленапросит ", "Елена просит "),
                    ("данорм ", "да норм "),
                ] {
                    let mut req = request(input, &pipeline, mode);
                    req.nanda_candidate_route = CandidateReadoutRoute::live_default();
                    req.correction_safety = correction_safety;

                    let resolution = resolve_text_correction(req);
                    assert_eq!(
                        resolution
                            .selected
                            .as_ref()
                            .map(|candidate| candidate.replacement.as_str()),
                        Some(expected),
                        "grounded boundary regression for {mode:?}/{correction_safety:?}, input={input:?}: {resolution:#?}"
                    );
                }
            }
        }
    }

    #[test]
    fn live_canonical_l2_field_applies_bounded_typo_plus_boundary_repair() {
        let pipeline = default_typing_assist_pipeline();
        let mut req = request("Готовь докуентыдля ", &pipeline, CorrectionMode::NandaOnly);
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();

        let resolution = resolve_text_correction(req);
        let selected = resolution
            .selected
            .as_ref()
            .expect("verified current-token repair plus split must remain in the live lattice");

        assert_eq!(selected.replacement, "Готовь документы для ");
        assert_eq!(selected.origin, CandidateOrigin::Boundary);
        assert_eq!(selected.error_class, TypingErrorClass::GluedWords);
        assert_eq!(selected.source_id, "CanonicalL2FieldBoundary");
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn live_canonical_l2_field_applies_trailing_short_function_boundary() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected) in [
            ("Готовь документыдля ", "Готовь документы для "),
            ("Какие документыим ", "Какие документы им "),
        ] {
            let mut req = request(input, &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();

            let resolution = resolve_text_correction(req);
            let selected = resolution
                .selected
                .as_ref()
                .unwrap_or_else(|| panic!("missing boundary decision: {resolution:#?}"));

            assert_eq!(selected.replacement, expected, "input={input:?}");
            assert_eq!(selected.origin, CandidateOrigin::Boundary);
            assert_eq!(selected.error_class, TypingErrorClass::GluedWords);
            assert_eq!(selected.source_id, "CanonicalL2FieldBoundary");
        }
    }

    #[test]
    fn live_l2_field_owner_blocks_reference_only_semantic_word_drift() {
        let pipeline = default_typing_assist_pipeline();
        for input in ["модель генерит ", "окончанием слов "] {
            let mut req = request(input, &pipeline, CorrectionMode::NandaOnly);
            req.nanda_candidate_route = CandidateReadoutRoute::live_default();
            let resolution = resolve_text_correction(req);

            assert_eq!(
                resolution.decision, None,
                "live owner must preserve an already valid phrase: {resolution:#?}"
            );
        }
    }

    #[test]
    fn td007_current_layout_autocorrect_requires_surface_authority() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected) in [("lfdfq ", "давай "), ("rfr ", "как ")] {
            let decision = decide_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ))
            .unwrap_or_else(|| panic!("wrong-layout candidate missing for {input:?}"));
            assert_eq!(decision.replacement, expected, "input={input:?}");
            assert_eq!(decision.source, CorrectionDecisionSource::Deterministic);
        }

        let unadmitted = "gthtdjhfxbdftncz ";
        assert_eq!(
            crate::dict::convert(unadmitted.trim(), crate::dict::Direction::Us2Ru),
            "переворачивается"
        );
        assert_eq!(
            decide_text_correction(request(
                unadmitted,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            )),
            None,
            "literal layout projection remains available without gaining autocorrect authority"
        );
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
    fn full_wave_does_not_append_a_typo_repair_to_layout_projection() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "djn nfrjt djn yt gthtdfhfxbdftncz ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.decision.is_none(), "resolution={resolution:#?}");
        assert!(
            resolution
                .candidates
                .iter()
                .all(|candidate| !candidate.has_source_id("LayoutSequenceCell32")),
            "resolution={resolution:#?}"
        );
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
            lexical_authority_frame: None,
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
    fn td007_current_missing_letter_routes_through_decision_and_verifier() {
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
        assert!(score.decision_rank_milli > 0);
        assert!(
            resolution
                .selected_transition
                .as_ref()
                .expect("selected transition receipt")
                .diagnostic_transition()
                .is_verified()
        );
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
    fn composite_label_cannot_substitute_for_boundary_transition_proof() {
        let gate = gate_candidate_with_source(
            "тоесть ",
            "то есть ",
            TypingErrorClass::CompositeTypo,
            ids::PERSONAL_PHRASE,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "edit_transition_not_verified");
    }

    #[test]
    fn boundary_shift_is_selected_by_the_unified_correction_core() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "я думаю допусти мнабираю ",
            &pipeline,
            CorrectionMode::NandaOnly,
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
            CorrectionMode::NandaOnly,
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
            let resolution =
                resolve_text_correction(request(text, &pipeline, CorrectionMode::NandaOnly));

            assert!(
                resolution.selected.as_ref().is_none_or(|candidate| {
                    candidate.error_class != TypingErrorClass::BoundaryShift
                }),
                "text={text:?} resolution={resolution:#?}"
            );
        }
    }

    #[test]
    fn verified_short_boundary_shift_applies_as_one_transition() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("во тты ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("resolution={resolution:#?}"));
        assert_eq!(selected.replacement, "вот ты ");
        assert_eq!(selected.error_class, TypingErrorClass::BoundaryShift);
        let transition = resolution
            .selected_transition
            .as_ref()
            .expect("verified boundary transition")
            .diagnostic_transition();
        assert!(transition.is_verified());
        assert_eq!(transition.changed_tokens(), Some(2));
    }

    #[test]
    fn ambiguous_long_l2_surface_drift_from_live_log_is_suggestion_only() {
        let pipeline = default_typing_assist_pipeline();
        let mut req = request(
            "самка схема парочинная ",
            &pipeline,
            CorrectionMode::NandaOnly,
        );
        req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let resolution = resolve_text_correction(req);

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        assert!(
            resolution
                .candidates
                .iter()
                .any(|candidate| candidate.replacement == "самка схема перочинная "),
            "resolution={resolution:#?}"
        );
        assert!(
            resolution.candidates.iter().any(|candidate| {
                candidate.replacement == "самка схема перочинная "
                    && candidate.gate.action == CandidateGateAction::SuggestOnly
            }),
            "resolution={resolution:#?}"
        );
    }

    #[test]
    fn split_phrase_candidate_wins_over_l2_shortcut() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("тоесть ", &pipeline, CorrectionMode::NandaOnly));

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
        let resolution =
            resolve_text_correction(request("приудишна ", &pipeline, CorrectionMode::NandaOnly));

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
    fn td007_p0_left_context_layout_repair_remains_suggest_only() {
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
    fn td007_current_layout_then_typo_keeps_typed_origin() {
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
        assert_eq!(selected.origin, CandidateOrigin::LayoutThenTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn td007_p0_known_english_word_does_not_flip_through_layout_fallback() {
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
    fn english_compare_reference_cannot_authorize_cross_script_projection_without_a_center() {
        let pipeline = default_typing_assist_pipeline();
        for original in ["dowenload ", "adress "] {
            let resolution =
                resolve_text_correction(request(original, &pipeline, CorrectionMode::NandaOnly));
            assert_eq!(
                resolution.decision,
                None,
                "input={original:?}: {resolution:#?}"
            );
            assert!(
                resolution.candidates.iter().all(|candidate| {
                    !matches!(
                        candidate.origin,
                        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
                    ) || candidate.gate.action != CandidateGateAction::Eligible
                }),
                "cross-script fallback gained authority for {original:?}: {resolution:#?}"
            );
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
            let mut req = request(original, &pipeline, CorrectionMode::NandaOnly);
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
            let mut req = request(original, &pipeline, CorrectionMode::NandaOnly);
            req.nanda_wave_options = req.nanda_wave_options.with_l2_phase_apply(true);
            let resolution = resolve_text_correction(req);
            assert!(
                resolution.selected.as_ref().is_none_or(|candidate| {
                    is_cyrillic_letters_only(candidate.replacement.trim())
                }),
                "known Russian center must retain script: {resolution:?}"
            );
        }
    }

    #[test]
    fn td007_p0_partial_typo_does_not_autoapply_unknown_intermediate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "помшник ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None, "resolution={resolution:#?}");
        assert!(
            resolution.candidates.iter().any(|candidate| {
                candidate.replacement == "помошник "
                    && candidate.error_class == TypingErrorClass::MissingLetter
                    && candidate.gate.action == CandidateGateAction::SuggestOnly
                    && candidate.gate.reason == "single_step_typo_still_unknown"
            }),
            "resolution={resolution:#?}"
        );
    }

    #[test]
    fn ambiguous_field_retains_single_step_repair_without_guessing() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "мы отвравим ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        assert!(resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "мы отравим "));
        assert!(resolution
            .decision
            .as_ref()
            .is_none_or(|decision| decision.replacement != "мы отвратим "));
    }

    #[test]
    fn deterministic_single_step_repair_keeps_its_owner() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "мы отвравим ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.as_ref().expect("selected candidate");
        assert_eq!(selected.replacement, "мы отравим ", "{resolution:?}");
        assert_eq!(selected.source, CorrectionDecisionSource::Deterministic);
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

        assert_eq!(
            decision.action,
            CandidateGateAction::Eligible,
            "{decision:?}"
        );
    }

    #[test]
    fn td007_current_l2_missing_letter_without_target_authority_is_suggestion_only() {
        let decision = gate_candidate_with_origin(
            "дожь ",
            "дождь ",
            TypingErrorClass::MissingLetter,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(decision.action, CandidateGateAction::SuggestOnly);
        assert_eq!(decision.reason, "known_current_word_surface_drift");
    }

    #[test]
    fn composite_typo_rejects_short_initial_consonant_growth() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "давай лушее ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(
            resolution
                .selected
                .as_ref()
                .is_none_or(|candidate| candidate.replacement != "давай глушее ")
        );
        assert!(resolution.candidates.iter().all(|candidate| {
            candidate.replacement != "давай глушее "
                || candidate.gate.action != CandidateGateAction::Eligible
        }));
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
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));

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
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));

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
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));

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
    fn local_typo_cases_follow_current_source_authority() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected, should_apply, suggest_reason) in [
            ("длеай ", Some("делай "), true, None),
            ("тарфик ", Some("трафик "), true, None),
            ("рабоатешь ", Some("работаешь "), true, None),
            (
                "агресивнее ",
                Some("агрессивнее "),
                false,
                Some("unproven_stable_surface_shape_drift"),
            ),
            ("дейстия ", Some("действия "), true, None),
            ("кнал ", Some("канал "), true, None),
            (
                "сбирать ",
                Some("собирать "),
                false,
                Some("known_current_word_surface_drift"),
            ),
            ("переспективнее ", None, false, None),
            ("отвликайся ", Some("отвлекайся "), true, None),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            let Some(expected) = expected else {
                assert!(
                    resolution.selected.is_none() && resolution.candidates.is_empty(),
                    "unowned deterministic case must fail closed for {input:?}: {resolution:?}"
                );
                continue;
            };
            let retained = resolution
                .candidates
                .iter()
                .find(|candidate| candidate.replacement == expected)
                .unwrap_or_else(|| panic!("retained candidate for {input:?}: {resolution:?}"));
            if should_apply {
                assert_eq!(
                    resolution.selected.as_ref(),
                    Some(retained),
                    "input={input:?}; {resolution:?}"
                );
                assert_eq!(retained.gate.action, CandidateGateAction::Eligible);
            } else {
                assert!(resolution.selected.is_none(), "input={input:?}: {resolution:?}");
                assert_eq!(retained.gate.action, CandidateGateAction::SuggestOnly);
                assert_eq!(retained.gate.reason, suggest_reason.expect("suggest reason"));
            }
        }
    }

    #[test]
    fn composite_typo_recovers_common_word_with_broken_prefix() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "где эсперемнт ",
            &pipeline,
            CorrectionMode::NandaOnly,
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
            CorrectionMode::NandaOnly,
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
    fn boundary_gate_admits_bounded_current_token_repair_and_split() {
        let gate = gate_candidate(
            "Готовь докуентыдля ",
            "Готовь документы для ",
            TypingErrorClass::GluedWords,
        );

        assert_eq!(gate.action, CandidateGateAction::Eligible);
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

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_rejects_short_function_split_with_unknown_tail() {
        let gate = gate_candidate("со скрина ", "со с крина ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_rejects_unproven_prefix_like_split() {
        for (original, replacement, expected_reason) in [
            ("перхвачу ", "пер хвачу ", "weak_boundary_split_tail"),
            (
                "внутренний ",
                "вну тренний ",
                "known_single_word_boundary_split",
            ),
            (
                "псевдоним ",
                "псев доним ",
                "known_single_word_boundary_split",
            ),
        ] {
            let gate = gate_candidate(original, replacement, TypingErrorClass::GluedWords);
            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "prefix fragment {original:?} -> {replacement:?}"
            );
            assert_eq!(gate.reason, expected_reason);
        }

        for (original, replacement) in [
            ("данорм ", "да норм "),
            ("тоесть ", "то есть "),
            ("Еленапросит ", "Елена просит "),
        ] {
            let gate = gate_candidate(original, replacement, TypingErrorClass::GluedWords);
            assert_eq!(
                gate.action,
                CandidateGateAction::Eligible,
                "valid boundary {original:?} -> {replacement:?}"
            );
        }
    }

    #[test]
    fn composite_typo_repairs_generated_russian_forms() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected, expected_class, should_apply) in [
            (
                "руских ",
                "русских ",
                TypingErrorClass::MissingLetter,
                false,
            ),
            (
                "звгрузи ",
                "загрузи ",
                TypingErrorClass::LetterSubstitution,
                true,
            ),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            let retained = resolution
                .candidates
                .iter()
                .find(|candidate| candidate.replacement == expected)
                .unwrap_or_else(|| panic!("retained candidate for {input:?}: {resolution:?}"));
            assert_eq!(retained.error_class, expected_class, "input={input:?}");
            if should_apply {
                assert_eq!(resolution.selected.as_ref(), Some(retained));
                assert_eq!(retained.gate.action, CandidateGateAction::Eligible);
            } else {
                assert!(resolution.selected.is_none());
                assert_eq!(retained.gate.action, CandidateGateAction::SuggestOnly);
                assert_eq!(
                    retained.gate.reason,
                    "unproven_stable_surface_shape_drift"
                );
            }
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
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));
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
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));

            assert_eq!(resolution.decision, None, "input={input:?}: {resolution:?}");
            assert!(
                resolution.selected.is_none(),
                "input={input:?}: {resolution:?}"
            );
        }
    }

    #[test]
    fn nanda_surface_candidates_from_logs_are_suggest_only_when_surface_is_weak() {
        for (input, replacement) in [("тели ", "тел "), ("нас моного ", "нас мюоного ")] {
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
        }
    }

    #[test]
    fn composite_typo_split_plus_tail_candidate_stays_suggest_only_without_semantic_proof() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ее простозальет свтеом ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let candidate = resolution
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "ее просто зальет светом ")
            .expect("the shape-only candidate must remain observable");
        assert_eq!(candidate.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            candidate.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(candidate.gate.action, CandidateGateAction::SuggestOnly);
        assert_ne!(
            resolution
                .selected
                .as_ref()
                .map(|selected| selected.replacement.as_str()),
            Some("ее просто зальет светом "),
            "an unrelated verified typo repair may win, but the shape-only split must not: {resolution:#?}"
        );
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
            let resolution =
                resolve_text_correction(request(input, &pipeline, CorrectionMode::NandaOnly));

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
            CorrectionMode::NandaOnly,
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
    fn td007_current_unique_nearest_typo_transition_can_apply() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ППОНИКАЕШЬ? ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("ПОНИКАЕШЬ? "),
            "resolution={resolution:#?}"
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.error_class),
            Some(TypingErrorClass::RepeatedLetter)
        );
        let selected = resolution.selected.as_ref().expect("selected candidate");
        let selected_word = last_text_word(&selected.replacement).expect("selected word");
        let selected_distance = damerau_levenshtein(
            &resolution.event.current_word.to_lowercase(),
            &selected_word.to_lowercase(),
        );
        assert_eq!(selected_distance, 1);
        assert_eq!(
            resolution
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.gate.action == CandidateGateAction::Eligible
                        && last_text_word(&candidate.replacement).is_some_and(|word| {
                            damerau_levenshtein(
                                &resolution.event.current_word.to_lowercase(),
                                &word.to_lowercase(),
                            ) == selected_distance
                        })
                })
                .count(),
            1,
            "nearest verified target must be unique: {resolution:#?}"
        );
        let selected_score = resolution
            .candidate_scores
            .iter()
            .find(|score| score.selected)
            .expect("selected score");
        assert!(selected_score.edit_transition_verified);
        assert!(!selected_score.edit_transition_left_context_changed);
        assert_eq!(selected_score.edit_transition_changed_tokens, 1);
        assert!(resolution
            .candidate_scores
            .iter()
            .filter(|score| !score.selected)
            .all(|score| selected_score.decision_rank_milli > score.decision_rank_milli));
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
        let mut req = request("в коде ", &pipeline, CorrectionMode::NandaOnly);
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
            CorrectionMode::NandaOnly,
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
        let mut req = request("давай там посмотри ", &pipeline, CorrectionMode::NandaOnly);
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

        let mut known_form_req = request("наполняют ", &pipeline, CorrectionMode::NandaOnly);
        known_form_req.nanda_candidate_route = CandidateReadoutRoute::live_default();
        let known_form_resolution = resolve_text_correction(known_form_req);
        assert_ne!(
            known_form_resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("наполняю "),
            "known -ять present form must not auto-rewrite: {known_form_resolution:#?}"
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
    fn compare_reference_retains_nonlocal_word_form_without_unverified_apply() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Азейбарджан ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        let retained = resolution
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "Азербайджан ")
            .unwrap_or_else(|| panic!("retained L2 word-form center: {resolution:#?}"));
        assert_eq!(retained.source, CorrectionDecisionSource::Nanda);
        assert!(retained.has_source_id("L2WordAttractorCell32"));
        assert_eq!(retained.error_class, TypingErrorClass::CompositeTypo);
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
            CorrectionMode::NandaOnly,
        ));

        let selected = resolution.selected.clone().unwrap_or_else(|| {
            panic!("selected default-route sparse omission center: {resolution:#?}")
        });
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
    fn compare_reference_retains_sparse_omission_without_unverified_apply() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько переподлчаю ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none(), "resolution={resolution:#?}");
        let retained = resolution
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "на сколько переподключаю ")
            .unwrap_or_else(|| panic!("retained sparse omission: {resolution:#?}"));
        assert_eq!(retained.source, CorrectionDecisionSource::Nanda);
        assert!(retained.has_source_id("L2WordAttractorCell32"));
        assert_eq!(
            retained.error_class,
            TypingErrorClass::SparseInternalMultiOmission
        );
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
            lexical_authority_frame: None,
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: false,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            nanda_candidate_route: CandidateReadoutRoute::FullWave,
            nanda_wave_options: WaveOptions::default(),
            mode: CorrectionMode::NandaOnly,
        });
        assert_eq!(decision, None);
    }
}
