
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

    fn push_candidates(self, req: &CorrectionRequest<'_>, lattice: &mut L2CandidateLattice) {
        match self {
            Self::Deterministic => lattice.push_source(deterministic_text_correction(req)),
            Self::Nanda => {
                for candidate in nanda_text_candidates(req) {
                    lattice.push_source(Some(candidate));
                }
            }
        }
    }
}
fn deterministic_text_correction(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !(req.auto_replace || req.typing_assist || req.auto_switch_layout) {
        return None;
    }
    if let Some(candidate) = multiword_layout_projection_candidate(req) {
        return Some(candidate);
    }

    let pipeline = typing_assist_pipeline_for_context(
        req.auto_replace,
        req.correction_safety,
        req.typing_assist_pipeline,
        req.text,
    );
    let explanation =
        explain_typing_assist_with_pipeline(req.text, req.auto_switch_layout, &pipeline);
    let Some(replacement) = explanation.output else {
        return deterministic_composite_text_correction(req, &pipeline);
    };
    let rule_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| candidate.rule_id.as_str())
        .unwrap_or("deterministic");
    let error_class = rule_error_class(rule_id);
    let gate = gate_candidate_with_source(req.text, &replacement, error_class, rule_id);
    if matches!(
        error_class,
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
    ) {
        if let Some(composite) = deterministic_composite_text_correction(req, &pipeline) {
            if should_prefer_composite_after_repeated_repair(
                req.text,
                &replacement,
                &composite.replacement,
            ) {
                return Some(composite);
            }
        }
    }
    if gate.action != CandidateGateAction::Apply {
        return deterministic_composite_text_correction(req, &pipeline).or(Some(
            UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Deterministic,
                rule_id,
                error_class,
                gate,
            ),
        ));
    }

    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        rule_id,
        error_class,
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
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::WrongLayout,
        ids::LAYOUT_EN_TO_RU,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        ids::LAYOUT_EN_TO_RU,
        TypingErrorClass::WrongLayout,
        gate,
    ))
}

fn deterministic_composite_text_correction(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    layout_then_typo_candidate(req, pipeline)
        .or_else(|| repeated_letter_fallback_candidate(req))
        .or_else(|| composite_russian_typo_candidate(req, pipeline))
}

fn short_cyrillic_layout_shadow_candidate(
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
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::WrongLayout,
        ids::LAYOUT_RU_TO_EN,
    );
    if gate.action != CandidateGateAction::SuggestOnly {
        return None;
    }

    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
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
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::RepeatedLetter,
        source_id,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
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
    let explanation = explain_typing_assist_with_pipeline(&converted_text, false, pipeline);
    if explanation.chosen.is_none() && is_protected_known_english_layout_word(&current_word) {
        return None;
    }
    let final_replacement = explanation.output.unwrap_or_else(|| {
        if crate::layout_autoswitch::is_russian_layout_surface_authority_word(&converted_word) {
            converted_text.clone()
        } else {
            String::new()
        }
    });
    if final_replacement.is_empty() || final_replacement == req.text {
        return None;
    }
    let source_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| format!("layout_then_{}", candidate.rule_id))
        .unwrap_or_else(|| "layout_then_known_word".to_string());
    let gate = gate_candidate_with_source(
        req.text,
        &final_replacement,
        TypingErrorClass::CompositeTypo,
        &source_id,
    );
    Some(UnifiedCorrectionCandidate::new(
        final_replacement,
        CorrectionDecisionSource::Deterministic,
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
    let field_knows_original =
        crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower)
        || crate::russian_lexicon::is_center_backed_russian_form(&lower);
    if !is_cyrillic_letters_only(&current_word) || field_knows_original {
        return None;
    }

    if let Some(replacement_word) = unique_adjacent_transposition_word(&current_word) {
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = ids::ADJACENT_TRANSPOSITION;
            let gate = gate_candidate_with_source(
                req.text,
                &replacement,
                TypingErrorClass::AdjacentTransposition,
                source_id,
            );
            return Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Deterministic,
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
            let gate = gate_candidate_with_source(
                req.text,
                &replacement,
                TypingErrorClass::CompositeTypo,
                source_id,
            );
            return Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Deterministic,
                source_id,
                TypingErrorClass::CompositeTypo,
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
            let typed_operator_bonus = if inserted_char_position_for_missing_letter(
                &lower,
                candidate,
            )
            .is_some()
            {
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
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::CompositeTypo,
        source_id,
    );
    let composite = UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
        source_id,
        TypingErrorClass::CompositeTypo,
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
    let rule_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| candidate.rule_id.as_str())
        .unwrap_or("current_word_rule");
    let error_class = rule_error_class(rule_id);
    let gate = gate_candidate_with_source(req.text, &replacement, error_class, rule_id);
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Deterministic,
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

fn nanda_text_candidates(req: &CorrectionRequest<'_>) -> Vec<UnifiedCorrectionCandidate> {
    if !req.nanda_autocorrect {
        return Vec::new();
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
        .previous_token_candidates(
            &words[..previous_index],
            observed_previous,
            next_token,
            4,
        )
        .into_iter()
        .filter(|candidate| candidate.support >= 2)
        .filter_map(|candidate| {
            let replacement = replace_penultimate_text_word(original, &candidate.token)?;
            let mut gate = gate_candidate_with_source(
                original,
                &replacement,
                TypingErrorClass::CompositeTypo,
                "PhraseMemoryCell32",
            );
            if matches!(
                gate.action,
                CandidateGateAction::Eligible | CandidateGateAction::Apply
            ) {
                gate = CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "delayed_context_requires_promotion",
                };
            }
            Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Nanda,
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
    let error_class = nanda_source_error_class(candidate.source);
    let gate = gate_candidate_with_source(original, &replacement, error_class, candidate.source);
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
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
        ids::MOVED_PREFIX_PAIR => TypingErrorClass::PartialLayout,
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

fn nanda_source_error_class(source: &str) -> TypingErrorClass {
    match correction_source_contract::source_role(source) {
        CorrectionSourceRole::Layout => TypingErrorClass::WrongLayout,
        CorrectionSourceRole::Boundary => TypingErrorClass::GluedWords,
        CorrectionSourceRole::Completion => TypingErrorClass::CompletionOnly,
        CorrectionSourceRole::L2Surface | CorrectionSourceRole::L3Context => {
            TypingErrorClass::CompositeTypo
        }
        CorrectionSourceRole::Technical => TypingErrorClass::TechnicalToken,
        CorrectionSourceRole::DeterministicTypo | CorrectionSourceRole::Unknown => match source {
            "ShortTokenCell32" => TypingErrorClass::PartialLayout,
            "GrammarCell32" => TypingErrorClass::GrammarAgreement,
            "CommonRuFixCell32" | "LearnedMemoryCell32" | "PhraseMemoryCell32" => {
                TypingErrorClass::CompositeTypo
            }
            _ => TypingErrorClass::Unknown,
        },
    }
}
