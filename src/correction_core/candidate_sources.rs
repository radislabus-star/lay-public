#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L2CandidateSource {
    Deterministic,
    Nanda,
}

impl L2CandidateSource {
    const DETERMINISTIC_ONLY: [Self; 1] = [Self::Deterministic];
    const NANDA_ONLY: [Self; 1] = [Self::Nanda];
    const DETERMINISTIC_THEN_NANDA: [Self; 2] = [Self::Deterministic, Self::Nanda];

    fn for_mode(mode: CorrectionMode) -> &'static [Self] {
        match mode {
            CorrectionMode::DeterministicOnly => &Self::DETERMINISTIC_ONLY,
            CorrectionMode::NandaOnly => &Self::NANDA_ONLY,
            CorrectionMode::DeterministicThenNanda => &Self::DETERMINISTIC_THEN_NANDA,
        }
    }

    fn push_candidates(
        self,
        req: &CorrectionRequest<'_>,
        lattice: &mut L2CandidateLattice,
        l2_peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
    ) {
        match self {
            Self::Deterministic => {
                lattice.extend_source(deterministic_text_candidates(req));
            }
            Self::Nanda => {
                let candidates =
                    if req.nanda_candidate_route == CandidateReadoutRoute::CanonicalL2Field {
                        let readout = crate::nanda_wave::l2_field::canonical_text_readout(req.text);
                        lattice.set_l2_field_authority(readout.authority);
                        readout.candidates
                    } else {
                        nanda_text_candidates(req, l2_peak_context)
                    };
                if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
                    eprintln!(
                        "candidate-lattice source=nanda count={} replacements={:?}",
                        candidates.len(),
                        candidates
                            .iter()
                            .map(|candidate| candidate.replacement.as_str())
                            .collect::<Vec<_>>()
                    );
                }
                lattice.extend_source(candidates);
            }
        }
    }
}

fn deterministic_text_candidates(req: &CorrectionRequest<'_>) -> Vec<UnifiedCorrectionCandidate> {
    let mut candidates = Vec::with_capacity(8);
    if let Some(candidate) = boundary_shift_transition_candidate(req) {
        candidates.push(candidate);
    }
    if !(req.auto_replace || req.typing_assist || req.auto_switch_layout) {
        return candidates;
    }
    if let Some(candidate) = multiword_layout_projection_candidate(req) {
        candidates.push(candidate);
    }

    let pipeline = typing_assist_pipeline_for_context(
        req.auto_replace,
        req.correction_safety,
        req.typing_assist_pipeline,
        req.text,
    );
    for (candidate, replacement) in
        collect_typing_assist_candidates_with_pipeline(req.text, req.auto_switch_layout, &pipeline)
    {
        let declared_error_class = rule_error_class(&candidate.rule_id);
        let origin = typing_rule_origin(candidate.score.family, declared_error_class);
        let error_class = action_operator::classify_token_transition(
            req.text,
            &replacement,
            origin,
            declared_error_class,
        );
        let gate = TransitionDecisionCore::admit_candidate_proposal(
            req.text,
            &replacement,
            error_class,
            origin,
        );
        candidates.push(UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Deterministic,
            origin,
            candidate.rule_id,
            error_class,
            gate,
        ));
    }
    candidates.extend(deterministic_composite_text_candidates(req, &pipeline));
    candidates
}

fn boundary_shift_transition_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.auto_replace && !req.typing_assist {
        return None;
    }
    let (leading, core, trailing) = split_edge_whitespace(req.text);
    let segments = split_ws_segments(core);
    let word_indices = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect::<Vec<_>>();
    let [.., left_idx, right_idx] = word_indices.as_slice() else {
        return None;
    };
    if *right_idx != *left_idx + 2 || !segments[*left_idx + 1].1 {
        return None;
    }

    let pair = format!(
        "{}{}{}",
        segments[*left_idx].0,
        segments[*left_idx + 1].0,
        segments[*right_idx].0
    );
    let pair_replacement = crate::phrase_reader::propose_moved_prefix_letter_pair(&pair)?;
    let mut replacement = String::with_capacity(req.text.len());
    replacement.push_str(leading);
    for (idx, (segment, _)) in segments.iter().enumerate() {
        if idx == *left_idx {
            replacement.push_str(&pair_replacement);
        } else if idx <= *right_idx && idx > *left_idx {
            continue;
        } else {
            replacement.push_str(segment);
        }
    }
    replacement.push_str(trailing);

    let origin = CandidateOrigin::Boundary;
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &replacement,
        TypingErrorClass::BoundaryShift,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        ids::MOVED_PREFIX_PAIR,
        TypingErrorClass::BoundaryShift,
        gate,
    ))
}

fn multiword_layout_projection_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.auto_switch_layout || req.text.split_whitespace().count() < 2 {
        return None;
    }
    let (leading, core, trailing) = split_edge_whitespace(req.text);
    let converted = crate::layout_autoswitch::correct_wrong_layout_ascii_phrase(core)?;
    let replacement = format!("{leading}{converted}{trailing}");
    let origin = CandidateOrigin::Layout;
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &replacement,
        TypingErrorClass::WrongLayout,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        ids::LAYOUT_EN_TO_RU,
        TypingErrorClass::WrongLayout,
        gate,
    ))
}

fn deterministic_composite_text_candidates(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Vec<UnifiedCorrectionCandidate> {
    [
        layout_then_typo_candidate(req, pipeline),
        repeated_letter_fallback_candidate(req),
        composite_russian_typo_candidate(req, pipeline),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn short_cyrillic_layout_suggestion_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.auto_switch_layout {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    if current_word.chars().count() > 2 || !has_cyrillic(&current_word) {
        return None;
    }

    let replacement_word = crate::dict::convert(&current_word, crate::dict::Direction::Ru2Us);
    if replacement_word == current_word
        || !replacement_word
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '`'))
    {
        return None;
    }

    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    let origin = CandidateOrigin::Layout;
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &replacement,
        TypingErrorClass::WrongLayout,
        origin,
    );
    if gate.action != CandidateGateAction::SuggestOnly {
        return None;
    }

    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        ids::LAYOUT_RU_TO_EN,
        TypingErrorClass::WrongLayout,
        gate,
    ))
}

fn repeated_letter_fallback_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.typing_assist && !req.auto_replace {
        return None;
    }
    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    let replacement_word = crate::ru_typo::correct_repeated_letter(&current_word)
        .or_else(|| unique_known_repeated_deletion_word(&current_word))?;
    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    if replacement == req.text || !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }

    let source_id = ids::REPEATED_LETTER;
    let origin = CandidateOrigin::DeterministicTypo;
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &replacement,
        TypingErrorClass::RepeatedLetter,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        source_id,
        TypingErrorClass::RepeatedLetter,
        gate,
    ))
}

fn unique_known_repeated_deletion_word(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    let mut candidates = repeated_run_deletion_candidates(&lower)
        .into_iter()
        .filter(|candidate| repeated_deletion_has_surface_support(&lower, candidate))
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(apply_word_case(word, candidate))
}

fn layout_then_typo_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    if !req.auto_switch_layout {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    if !looks_like_ascii_layout_word(&current_word) {
        return None;
    }

    let converted_word = crate::dict::convert(&current_word, crate::dict::Direction::Us2Ru);
    if converted_word == current_word || !is_cyrillic_letters_only(&converted_word) {
        return None;
    }

    let converted_text = replace_last_text_word(req.text, &converted_word)?;
    let raw_projection_stable =
        crate::layout_autoswitch::is_russian_layout_surface_authority_word(&converted_word);
    let (final_replacement, source_id) = if raw_projection_stable {
        (converted_text, "layout_then_known_word".to_string())
    } else {
        let explanation = explain_typing_assist_with_pipeline(&converted_text, false, pipeline);
        if explanation.chosen.is_none() && is_protected_known_english_layout_word(&current_word) {
            return None;
        }
        let source_id = explanation
            .chosen
            .as_ref()
            .map(|candidate| format!("layout_then_{}", candidate.rule_id))
            .unwrap_or_else(|| "layout_then_unknown_word".to_string());
        (explanation.output.unwrap_or_default(), source_id)
    };
    if final_replacement.is_empty() || final_replacement == req.text {
        return None;
    }
    let origin = CandidateOrigin::LayoutThenTypo;
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &final_replacement,
        TypingErrorClass::CompositeTypo,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        final_replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        source_id,
        TypingErrorClass::CompositeTypo,
        gate,
    ))
}

fn is_protected_known_english_layout_word(word: &str) -> bool {
    crate::layout_autoswitch::is_protected_ascii_layout_token(word)
        && crate::layout_autoswitch::is_known_english_layout_autoswitch_word(
            &word.to_ascii_lowercase(),
        )
}

fn composite_russian_typo_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    if !req.typing_assist && !req.auto_replace {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    let lower = current_word.to_lowercase();
    let field_knows_original = crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower)
        || crate::russian_lexicon::is_center_backed_russian_form(&lower);
    if !is_cyrillic_letters_only(&current_word) || field_knows_original {
        return None;
    }

    if let Some(replacement_word) = unique_adjacent_transposition_word(&current_word) {
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = ids::ADJACENT_TRANSPOSITION;
            let origin = CandidateOrigin::DeterministicTypo;
            let gate = TransitionDecisionCore::admit_candidate_proposal(
                req.text,
                &replacement,
                TypingErrorClass::AdjacentTransposition,
                origin,
            );
            return Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Deterministic,
                origin,
                source_id,
                TypingErrorClass::AdjacentTransposition,
                gate,
            ));
        }
    }

    if lower.chars().count() < 4 {
        return None;
    }

    if let Some(candidate) = repeated_prefix_composite_word(&lower) {
        let replacement_word = apply_word_case(&current_word, &candidate);
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = "composite_ru_typo";
            let origin = CandidateOrigin::DeterministicTypo;
            let error_class = action_operator::classify_token_transition(
                req.text,
                &replacement,
                origin,
                TypingErrorClass::CompositeTypo,
            );
            let gate = TransitionDecisionCore::admit_candidate_proposal(
                req.text,
                &replacement,
                error_class,
                origin,
            );
            return Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Deterministic,
                origin,
                source_id,
                error_class,
                gate,
            ));
        }
    }

    let single_step = current_word_rule_candidate(req, pipeline, &current_word);
    let Some((candidate, _)) = crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(&lower),
        0.85,
        |candidate| {
            if candidate == &lower
                || repeated_run_deletion_candidates(&lower)
                    .iter()
                    .any(|repaired| repaired == candidate)
                || !crate::russian_lexicon::is_known_russian_word_or_form(candidate)
            {
                return None;
            }
            let distance = damerau_levenshtein(&lower, candidate);
            if distance == 0 || distance > 3 {
                return None;
            }
            let inserted = candidate
                .chars()
                .count()
                .saturating_sub(lower.chars().count());
            if risky_short_initial_insertion(&lower, candidate) {
                return None;
            }
            let common_word_recovery = lower.chars().count() >= 7
                && inserted <= 2
                && distance <= 3
                && repeated_run_deletion_candidates(&lower).is_empty()
                && crate::lexicon::is_common_ru_word(candidate);
            if !common_word_recovery
                && !compatible_composite_typo_shape(&lower, candidate, distance)
            {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, &lower);
            if inserted > 1 && !common_word_recovery {
                return None;
            }
            let shape_bonus = inserted as f64 * 8.0;
            let typed_operator_bonus =
                if inserted_char_position_for_missing_letter(&lower, candidate).is_some() {
                    10.0
                } else {
                    0.0
                };
            let close_insert_bonus = if distance == 1 && inserted == 1 {
                12.0
            } else {
                0.0
            };
            let common_word_bonus = if common_word_recovery { 12.0 } else { 0.0 };
            let repeated_repair_bonus =
                if repeated_prefix_typo_shape_is_preserved(&lower, candidate) {
                    8.0
                } else {
                    0.0
                };
            let initial_vowel_bonus =
                missing_initial_vowel_before_double_consonant_bonus(&lower, candidate);
            let score = margin
                + shape_bonus
                + typed_operator_bonus
                + close_insert_bonus
                + common_word_bonus
                + repeated_repair_bonus
                + initial_vowel_bonus
                - distance as f64 * 0.35;
            (score >= 0.0).then_some(score)
        },
    ) else {
        return single_step;
    };
    let replacement_word = apply_word_case(&current_word, &candidate);
    let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
        .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
    if replacement == req.text {
        return None;
    }
    if !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }

    let source_id = "composite_ru_typo";
    let origin = CandidateOrigin::DeterministicTypo;
    let error_class = action_operator::classify_token_transition(
        req.text,
        &replacement,
        origin,
        TypingErrorClass::CompositeTypo,
    );
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &replacement,
        error_class,
        origin,
    );
    let composite = UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        source_id,
        error_class,
        gate,
    );
    if let Some(single_step) = single_step {
        if !should_prefer_composite_after_repeated_repair(
            req.text,
            &single_step.replacement,
            &composite.replacement,
        ) {
            return Some(single_step);
        }
    }
    Some(composite)
}

fn current_word_rule_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
    current_word: &str,
) -> Option<UnifiedCorrectionCandidate> {
    let current_tail = format!("{current_word} ");
    let explanation = explain_typing_assist_with_pipeline(&current_tail, false, pipeline);
    let replacement_tail = explanation.output?;
    let replacement_word = last_text_word(&replacement_tail)?;
    if replacement_word == current_word || !is_cyrillic_letters_only(&replacement_word) {
        return None;
    }
    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    if replacement == req.text || !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }
    let (rule_id, family) = explanation
        .chosen
        .as_ref()
        .map(|candidate| (candidate.rule_id.as_str(), candidate.score.family))
        .unwrap_or(("current_word_rule", TypingCandidateFamily::Unknown));
    let error_class = rule_error_class(rule_id);
    let origin = typing_rule_origin(family, error_class);
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        req.text,
        &replacement,
        error_class,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        origin,
        rule_id,
        error_class,
        gate,
    ))
}

fn repeated_prefix_composite_word(lower: &str) -> Option<String> {
    let repaired_forms = repeated_run_deletion_candidates(lower);
    if repaired_forms.is_empty() {
        return None;
    }
    crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(lower)
            .into_iter()
            .filter(|candidate| {
                candidate != lower
                    && !repaired_forms.iter().any(|repaired| repaired == candidate)
                    && crate::russian_lexicon::is_known_russian_word_or_form(candidate)
                    && repaired_forms
                        .iter()
                        .any(|repaired| damerau_levenshtein(repaired, candidate) <= 1)
            }),
        0.25,
        |candidate| {
            let best_repaired_distance = repaired_forms
                .iter()
                .map(|repaired| damerau_levenshtein(repaired, candidate))
                .min()?;
            if best_repaired_distance > 1 {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, lower);
            Some(margin + 8.0 - best_repaired_distance as f64 * 0.35)
        },
    )
    .map(|(candidate, _)| candidate)
}

fn nanda_text_candidates(
    req: &CorrectionRequest<'_>,
    l2_peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
) -> Vec<UnifiedCorrectionCandidate> {
    if !req.nanda_autocorrect {
        return Vec::new();
    }

    nanda_text_candidates_for_route(req, req.nanda_candidate_route, l2_peak_context)
}

fn nanda_text_candidates_for_route(
    req: &CorrectionRequest<'_>,
    route: CandidateReadoutRoute,
    _l2_peak_context: Option<&crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext>,
) -> Vec<UnifiedCorrectionCandidate> {
    match route {
        CandidateReadoutRoute::CanonicalL2Field => {
            return crate::nanda_wave::l2_field::canonical_text_candidates(req.text);
        }
        CandidateReadoutRoute::FullWave => {}
    }

    let trace = run_wave_trace_with_options(req.text, &req.nanda_wave_options);
    let mut candidates = trace
        .l2_candidates
        .iter()
        .filter_map(|candidate| nanda_word_candidate(req.text, candidate))
        .collect::<Vec<_>>();
    candidates.extend(delayed_context_candidates(req.text));
    candidates
}

fn delayed_context_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    if !original.ends_with(char::is_whitespace)
        || !crate::nanda_wave::llmwave::default_memory_is_warm()
    {
        return Vec::new();
    }
    crate::nanda_wave::llmwave::with_default_memory(|memory| {
        delayed_context_candidates_with_memory(original, memory)
    })
}

#[cfg(test)]
mod candidate_sources_tests {
    use super::*;
    use crate::config::{default_typing_assist_pipeline, CorrectionSafety};
    use std::fs;
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::thread;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    fn with_l11_socket_env<T>(socket_path: &Path, f: impl FnOnce() -> T) -> T {
        let _lock = env_lock();
        let previous = std::env::var_os("LAY_L11_SOCKET");
        std::env::set_var("LAY_L11_SOCKET", socket_path);
        let output = f();
        match previous {
            Some(value) => std::env::set_var("LAY_L11_SOCKET", value),
            None => std::env::remove_var("LAY_L11_SOCKET"),
        }
        output
    }

    fn with_l11_socket_env_cleared<T>(f: impl FnOnce() -> T) -> T {
        let _lock = env_lock();
        let previous = std::env::var_os("LAY_L11_SOCKET");
        std::env::remove_var("LAY_L11_SOCKET");
        let output = f();
        match previous {
            Some(value) => std::env::set_var("LAY_L11_SOCKET", value),
            None => std::env::remove_var("LAY_L11_SOCKET"),
        }
        output
    }

    fn temp_socket_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lay-{name}-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ))
    }

    fn spawn_mock_l11_service(
        response: crate::nanda_wave::L1ServiceResponse,
    ) -> (PathBuf, thread::JoinHandle<()>) {
        let socket_path = temp_socket_path("l11-mock");
        let listener = UnixListener::bind(&socket_path).expect("bind mock socket");
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept mock request");
            let mut reader = BufReader::new(stream.try_clone().expect("clone mock stream"));
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).expect("read request");
            let request: crate::nanda_wave::L1ServiceRequest =
                serde_json::from_str(line.trim_end()).expect("decode request");
            assert!(matches!(
                request,
                crate::nanda_wave::L1ServiceRequest::Lattice { .. }
            ));
            serde_json::to_writer(&mut writer, &response).expect("encode response");
            writer.write_all(b"\n").expect("write newline");
            writer.flush().expect("flush response");
        });
        (socket_path, handle)
    }

    fn request<'a>(text: &'a str, pipeline: &'a [TypingAssistRuleConfig]) -> CorrectionRequest<'a> {
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
            mode: CorrectionMode::DeterministicThenNanda,
        }
    }

    fn resolve_for_route<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
        route: CandidateReadoutRoute,
    ) -> crate::correction_core::CorrectionResolution {
        let mut req = request(text, pipeline);
        req.nanda_candidate_route = route;
        crate::correction_core::resolve_text_correction(req)
    }

    #[test]
    fn boundary_shift_source_keeps_tail_pair_eligible() {
        let pipeline = default_typing_assist_pipeline();
        let candidate =
            boundary_shift_transition_candidate(&request("я думаю допусти мнабираю ", &pipeline))
                .expect("boundary candidate");

        assert_eq!(candidate.replacement, "я думаю допустим набираю ");
        assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn deterministic_candidates_keep_verified_boundary_eligible() {
        let pipeline = default_typing_assist_pipeline();
        let req = request("я думаю допусти мнабираю ", &pipeline);
        let candidates = deterministic_text_candidates(&req);
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.replacement == "я думаю допустим набираю ")
            .expect("boundary candidate");

        assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn canonical_l2_field_route_uses_owned_surface_source_ids() {
        let candidates = with_l11_socket_env_cleared(|| {
            nanda_text_candidates_for_route(
                &request("пукнт ", &default_typing_assist_pipeline()),
                CandidateReadoutRoute::CanonicalL2Field,
                None,
            )
        });

        assert!(
            candidates
                .iter()
                .any(|candidate| {
                    candidate.source_id == "CanonicalL2FieldSurface"
                        || candidate
                            .evidence
                            .iter()
                            .any(|evidence| evidence.source_id == "CanonicalL2FieldSurface")
                }),
            "canonical route must self-birth surface candidates inside its owned local field: {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source_id.starts_with("CanonicalL2Field")),
            "canonical route must emit only canonical L2 source ids: {candidates:?}"
        );
    }

    #[test]
    fn canonical_l2_field_self_prepares_l11_candidate_without_peak_context() {
        let response = crate::nanda_wave::L1ServiceResponse::Lattice {
            seeds: vec![crate::nanda_wave::L11SeedSurface {
                terminal_id: Some(1),
                surface: "время".to_string(),
                authority: true,
                score_milli: 991,
            }],
        };
        let (socket_path, handle) = spawn_mock_l11_service(response);

        let candidates = with_l11_socket_env(&socket_path, || {
            nanda_text_candidates_for_route(
                &request("врмея ", &default_typing_assist_pipeline()),
                CandidateReadoutRoute::CanonicalL2Field,
                None,
            )
        });
        handle.join().expect("mock service");
        let _ = fs::remove_file(&socket_path);

        let candidate = candidates
            .iter()
            .find(|candidate| candidate.replacement == "время ")
            .expect("canonical field should internalize authoritative L1.1 seed");
        assert!(
            matches!(
                candidate.source_id.as_str(),
                "CanonicalL2FieldSurface" | "CanonicalL2FieldReadout"
            ),
            "authoritative L1.1 seed must enter the owned L2 field, not survive as sidecar: {candidate:?}"
        );
        assert!(
            candidate
                .evidence
                .iter()
                .any(|evidence| evidence.source_id == "CanonicalL2FieldSurface"),
            "authoritative L1.1 seed must still carry owned surface provenance: {candidate:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source_id != "CanonicalL2FieldL11"),
            "canonical route should internalize L1.1 seed into the field, not emit a separate sidecar: {candidates:?}"
        );
    }

    #[test]
    fn canonical_l2_field_keeps_nonleader_neighbor_regressions_unselected() {
        for input in ["докурчиват ", "ЯДРА ", "ене ", "слои "] {
            let pipeline = default_typing_assist_pipeline();
            let reference = resolve_for_route(input, &pipeline, CandidateReadoutRoute::FullWave);
            let canonical =
                resolve_for_route(input, &pipeline, CandidateReadoutRoute::CanonicalL2Field);
            let reference_selected = reference.selected.as_ref().map(|candidate| {
                (
                    candidate.replacement.as_str(),
                    candidate.gate.action,
                    candidate.gate.reason,
                    candidate.error_class,
                )
            });
            let canonical_selected = canonical.selected.as_ref().map(|candidate| {
                (
                    candidate.replacement.as_str(),
                    candidate.gate.action,
                    candidate.gate.reason,
                    candidate.error_class,
                )
            });

            assert_eq!(
                canonical_selected, reference_selected,
                "canonical selected parity changed for {input:?}\nreference={:?}\ncanonical={:?}",
                reference.selected, canonical.selected
            );
            assert!(
                !canonical
                    .candidates
                    .iter()
                    .any(|candidate| candidate.source_id == "CanonicalL2FieldReadout"),
                "canonical local readout must not collapse nonleader field for {input:?}: {:?}",
                canonical.candidates
            );
        }
    }

    #[test]
    fn canonical_l2_field_preserves_surface_parity_when_local_readout_abstains() {
        for input in ["смеа ", "сли, ", "вошеьные "] {
            let pipeline = default_typing_assist_pipeline();
            let reference = resolve_for_route(input, &pipeline, CandidateReadoutRoute::FullWave);
            let canonical =
                resolve_for_route(input, &pipeline, CandidateReadoutRoute::CanonicalL2Field);
            let reference_selected = reference
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str());
            let canonical_selected = canonical
                .selected
                .as_ref()
                .map(|candidate| candidate.replacement.as_str());

            assert_eq!(
                canonical_selected, reference_selected,
                "canonical selected surface parity changed for {input:?}\nreference={:?}\ncanonical={:?}",
                reference.selected, canonical.selected
            );
            assert!(
                canonical
                    .selected
                    .as_ref()
                    .is_none_or(|candidate| candidate.source_id == "CanonicalL2FieldSurface"),
                "canonical field should stay on surface owner when local readout abstains for {input:?}: {:?}",
                canonical.selected
            );
        }
    }

    #[test]
    fn l2_field_births_generic_short_layout_candidate_for_l3_context() {
        let candidates = with_l11_socket_env_cleared(|| {
            nanda_text_candidates_for_route(
                &request("Apple b ", &default_typing_assist_pipeline()),
                CandidateReadoutRoute::CanonicalL2Field,
                None,
            )
        });
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.replacement == "Apple и ")
            .expect("L2 field must preserve the exact b -> и layout projection");

        assert_eq!(candidate.source_id, "CanonicalL2FieldSurface");
        assert_eq!(candidate.origin, CandidateOrigin::Layout);
        assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn unresolved_short_layout_lattice_abstains_without_l3_context_authority() {
        let pipeline = default_typing_assist_pipeline();
        for input in ["Apple b ", "wave b ", "a b ", "b "] {
            let resolution =
                resolve_for_route(input, &pipeline, CandidateReadoutRoute::CanonicalL2Field);
            assert!(
                resolution.selected.is_none(),
                "unsafe short layout authority for {input:?}: {resolution:?}"
            );
        }
    }
}

fn delayed_context_candidates_with_memory(
    original: &str,
    memory: &crate::nanda_wave::llmwave::LlmWaveMemory,
) -> Vec<UnifiedCorrectionCandidate> {
    let words = normalized_correction_words(original);
    if words.len() < 2 {
        return Vec::new();
    }
    let previous_index = words.len() - 2;
    let observed_previous = &words[previous_index];
    let next_token = &words[previous_index + 1];
    memory
        .previous_token_candidates(&words[..previous_index], observed_previous, next_token, 4)
        .into_iter()
        .filter(|candidate| candidate.support >= 2)
        .filter_map(|candidate| {
            let replacement = replace_penultimate_text_word(original, &candidate.token)?;
            let origin = CandidateOrigin::L3Context;
            let mut gate = TransitionDecisionCore::admit_candidate_proposal(
                original,
                &replacement,
                TypingErrorClass::CompositeTypo,
                origin,
            );
            if gate.action == CandidateGateAction::Eligible {
                gate = CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "delayed_context_requires_promotion",
                };
            }
            Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Nanda,
                origin,
                "PhraseMemoryCell32",
                TypingErrorClass::CompositeTypo,
                gate,
            ))
        })
        .collect()
}

fn replace_penultimate_text_word(text: &str, replacement_word: &str) -> Option<String> {
    let (leading_ws, core, trailing_ws) = split_edge_whitespace(text);
    let segments = split_ws_segments(core);
    let word_indices = segments
        .iter()
        .enumerate()
        .filter_map(|(index, (segment, is_ws))| {
            if *is_ws {
                return None;
            }
            let (_, word, _) = split_word_punctuation(segment);
            (!word.is_empty()).then_some(index)
        })
        .collect::<Vec<_>>();
    let [.., previous_index, _current_index] = word_indices.as_slice() else {
        return None;
    };
    let (prefix, previous_word, suffix) = split_word_punctuation(segments[*previous_index].0);
    if previous_word.is_empty() {
        return None;
    }
    let replacement_word = apply_word_case(previous_word, replacement_word);
    let mut output = String::with_capacity(text.len() + replacement_word.len());
    output.push_str(leading_ws);
    for (index, (segment, _)) in segments.iter().enumerate() {
        if index == *previous_index {
            output.push_str(prefix);
            output.push_str(&replacement_word);
            output.push_str(suffix);
        } else {
            output.push_str(segment);
        }
    }
    output.push_str(trailing_ws);
    (output != text).then_some(output)
}

fn nanda_word_candidate(
    original: &str,
    candidate: &WordCandidate,
) -> Option<UnifiedCorrectionCandidate> {
    let replacement = preserve_candidate_trailing_separator(original, &candidate.text);
    if replacement == original {
        return None;
    }
    let origin = candidate.origin;
    let error_class = action_operator::classify_token_transition(
        original,
        &replacement,
        origin,
        TypingErrorClass::Unknown,
    );
    if candidate.source == "BoundaryCell32"
        && origin == CandidateOrigin::Boundary
        && matches!(
            error_class,
            TypingErrorClass::GluedWords | TypingErrorClass::SplitWord
        )
        && !crate::text_metrics::current_token_boundary_split_or_repair(original, &replacement)
    {
        return None;
    }
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        original,
        &replacement,
        error_class,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
        origin,
        candidate.source,
        error_class,
        gate,
    ))
}

fn preserve_candidate_trailing_separator(original: &str, candidate: &str) -> String {
    let mut out = candidate.to_string();
    if original
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
        && !out.chars().next_back().is_some_and(char::is_whitespace)
    {
        out.push(' ');
    }
    out
}

fn unique_adjacent_transposition_word(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_letters_only(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return None;
    }
    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;

    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }

        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        if candidate == lower || !crate::russian_lexicon::is_known_russian_word_or_form(&candidate)
        {
            continue;
        }
        if crate::ngram::ru_candidate_margin(&candidate, &lower) < COMPOSITE_TRANSPOSE_MIN_MARGIN {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn replace_last_word_and_split_previous_glued(
    text: &str,
    replacement_word: &str,
) -> Option<String> {
    let (leading_ws, core, trailing_ws) = split_edge_whitespace(text);
    let segments = split_ws_segments(core);
    let word_indices: Vec<usize> = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect();
    let [.., prev_idx, last_idx] = word_indices.as_slice() else {
        return None;
    };

    let (prev_leading, prev_word, prev_trailing) = split_word_punctuation(segments[*prev_idx].0);
    let (last_leading, last_word, last_trailing) = split_word_punctuation(segments[*last_idx].0);
    if !prev_leading.is_empty()
        || !prev_trailing.is_empty()
        || !last_leading.is_empty()
        || prev_word.is_empty()
        || last_word.is_empty()
        || !is_cyrillic_letters_only(prev_word)
        || !is_cyrillic_letters_only(last_word)
    {
        return None;
    }

    let split_previous = split_previous_glued_before_typo(prev_word)?;
    let mut output = String::with_capacity(text.len() + replacement_word.len() + 1);
    output.push_str(leading_ws);
    for (idx, (segment, _is_ws)) in segments.iter().enumerate() {
        if idx == *prev_idx {
            output.push_str(&split_previous);
        } else if idx == *last_idx {
            output.push_str(replacement_word);
            output.push_str(last_trailing);
        } else {
            output.push_str(segment);
        }
    }
    output.push_str(trailing_ws);

    (output != text).then_some(output)
}

fn split_previous_glued_before_typo(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    if lower.chars().count() < 8 || crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return None;
    }

    let mut candidates = Vec::new();
    for split in cyrillic_word_splits(&lower) {
        if split.left_len < 4 || split.left_len > 8 || split.right_len < 5 {
            continue;
        }
        if !crate::phrase_lexicon::is_known_russian_phrase_part(split.left) {
            continue;
        }
        if !crate::phrase_lexicon::is_known_russian_phrase_part(split.right)
            && !looks_like_contextual_russian_verb_form(split.right)
        {
            continue;
        }

        let candidate = format!("{} {}", split.left, split.right);
        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if margin < -30.0 {
            continue;
        }
        let verb_bonus = if looks_like_contextual_russian_verb_form(split.right) {
            2.0
        } else {
            0.0
        };
        candidates.push((candidate, margin + verb_bonus));
    }

    let ((candidate, _), _) =
        crate::candidate_ranker::choose_best_with_gap(candidates, 0.75, |(_, score)| Some(*score))?;
    Some(crate::text_case::apply_phrase_case(word, &candidate))
}

fn looks_like_contextual_russian_verb_form(word: &str) -> bool {
    word.chars().count() >= 5
        && [
            "ается",
            "яется",
            "уется",
            "ется",
            "етесь",
            "итесь",
            "аешь",
            "яешь",
            "уешь",
            "ешь",
            "ишь",
            "аете",
            "яете",
            "уете",
            "аете",
            "яете",
            "ует",
            "ает",
            "яет",
            "ете",
            "ите",
            "ают",
            "яют",
            "уют",
            "ет",
            "ит",
            "ут",
            "ют",
            "ат",
            "ят",
        ]
        .iter()
        .any(|ending| word.ends_with(ending))
}

fn looks_like_ascii_layout_word(word: &str) -> bool {
    word.is_ascii()
        && word.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 3
        && !word.chars().any(|ch| ch.is_ascii_digit())
        && word.chars().all(|ch| {
            ch.is_ascii_alphabetic()
                || matches!(
                    ch,
                    '\'' | ';'
                        | '['
                        | ']'
                        | '`'
                        | ','
                        | '.'
                        | '-'
                        | '{'
                        | '}'
                        | ':'
                        | '"'
                        | '<'
                        | '>'
                        | '~'
                )
        })
}

fn loose_original_shape_is_preserved(original: &str, candidate: &str) -> bool {
    if candidate.chars().count() < original.chars().count() {
        return false;
    }

    let mut original_chars = original.chars().map(loose_shape_char);
    let mut needed = original_chars.next();
    for ch in candidate.chars().map(loose_shape_char) {
        if needed == Some(ch) {
            needed = original_chars.next();
            if needed.is_none() {
                return true;
            }
        }
    }
    needed.is_none()
}

fn compatible_composite_typo_shape(original: &str, candidate: &str, distance: usize) -> bool {
    if loose_original_shape_is_preserved(original, candidate) {
        return true;
    }
    if repeated_prefix_typo_shape_is_preserved(original, candidate) {
        return true;
    }

    distance == 1 && original.chars().count() == candidate.chars().count()
}

fn missing_initial_vowel_before_double_consonant_bonus(original: &str, candidate: &str) -> f64 {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, candidate)
    else {
        return 0.0;
    };
    if idx != 0 {
        return 0.0;
    }
    let mut chars = original.chars();
    let Some(first) = chars.next() else {
        return 0.0;
    };
    if chars.next() != Some(first) {
        return 0.0;
    }
    match inserted {
        'э' => 10.0,
        'а' | 'о' | 'и' | 'у' | 'е' | 'ё' | 'ю' | 'я' => 2.0,
        _ => 0.0,
    }
}

fn risky_short_initial_insertion(original: &str, candidate: &str) -> bool {
    if original.chars().count() > 6 {
        return false;
    }
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, candidate)
    else {
        return false;
    };
    idx == 0 && (!crate::russian_chars::is_russian_vowel(inserted) || original.chars().count() <= 6)
}

fn repeated_prefix_typo_shape_is_preserved(original: &str, candidate: &str) -> bool {
    repeated_run_deletion_candidates(original)
        .into_iter()
        .any(|repaired| {
            if loose_original_shape_is_preserved(&repaired, candidate) {
                return true;
            }
            repaired.chars().count() == candidate.chars().count()
                && damerau_levenshtein(&repaired, candidate) <= 1
        })
}

fn loose_shape_char(ch: char) -> char {
    match ch {
        'ё' => 'е',
        'Ё' => 'Е',
        'щ' => 'ш',
        'Щ' => 'Ш',
        other => other,
    }
}

fn rule_error_class(rule_id: &str) -> TypingErrorClass {
    match rule_id {
        ids::MIXED_SCRIPT_LAYOUT | ids::DUPLICATE_LAYOUT_PREFIX => TypingErrorClass::MixedScript,
        ids::LAYOUT_TECHNICAL => TypingErrorClass::TechnicalToken,
        ids::FAST_LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_RU_CONJUNCTION_I
        | ids::CONTEXTUAL_RU_PREPOSITION_V
        | ids::LAYOUT_RU_TO_EN
        | ids::LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_RU_TO_EN
        | ids::VISUAL_B => TypingErrorClass::WrongLayout,
        ids::MOVED_PREFIX_PAIR => TypingErrorClass::BoundaryShift,
        ids::SPLIT_WORD_PAIR => TypingErrorClass::SplitWord,
        ids::CYRILLIC_CASE => TypingErrorClass::CaseNoise,
        ids::HARD_SIGN | ids::SINGLE_LETTER_SUBSTITUTION | ids::VOWEL_CONFUSION => {
            TypingErrorClass::LetterSubstitution
        }
        ids::ADJACENT_TRANSPOSITION => TypingErrorClass::AdjacentTransposition,
        ids::REPEATED_LETTER => TypingErrorClass::RepeatedLetter,
        ids::EXTRA_LETTERS => TypingErrorClass::ExtraLetter,
        ids::MISSING_LETTER => TypingErrorClass::MissingLetter,
        ids::VERB_ENDING => TypingErrorClass::GrammarAgreement,
        ids::GLUED_PHRASE => TypingErrorClass::GluedWords,
        ids::PERSONAL_PHRASE | ids::PERSONAL_TOKEN => TypingErrorClass::CompositeTypo,
        _ => TypingErrorClass::Unknown,
    }
}

fn typing_rule_origin(
    family: TypingCandidateFamily,
    error_class: TypingErrorClass,
) -> CandidateOrigin {
    match error_class {
        TypingErrorClass::WrongLayout
        | TypingErrorClass::PartialLayout
        | TypingErrorClass::MixedScript => CandidateOrigin::Layout,
        TypingErrorClass::BoundaryShift
        | TypingErrorClass::SplitWord
        | TypingErrorClass::GluedWords => CandidateOrigin::Boundary,
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
            CandidateOrigin::Technical
        }
        _ => match family {
            TypingCandidateFamily::Layout => CandidateOrigin::Layout,
            TypingCandidateFamily::Structural => CandidateOrigin::Boundary,
            TypingCandidateFamily::Exact
            | TypingCandidateFamily::Visual
            | TypingCandidateFamily::Typo
            | TypingCandidateFamily::Cleanup
            | TypingCandidateFamily::Unknown => CandidateOrigin::DeterministicTypo,
        },
    }
}
