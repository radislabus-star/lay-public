use super::*;

#[cfg(test)]
pub(super) fn gate_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> CandidateGateDecision {
    gate_candidate_with_origin(
        original,
        replacement,
        error_class,
        CandidateOrigin::DeterministicTypo,
    )
}

/// Compatibility adapter for legacy fixtures. Runtime producers must supply a
/// typed origin and cannot route authority through a diagnostic source name.
#[cfg(test)]
pub(super) fn gate_candidate_with_source(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CandidateGateDecision {
    gate_candidate_with_origin(
        original,
        replacement,
        error_class,
        fixture_origin(source_id),
    )
}

#[cfg(test)]
fn fixture_origin(source_id: &str) -> CandidateOrigin {
    if source_id.starts_with("layout_then_") {
        CandidateOrigin::LayoutThenTypo
    } else if source_id.contains("layout")
        || matches!(source_id, "LayoutWordCell32" | "ShortTokenCell32")
    {
        CandidateOrigin::Layout
    } else if matches!(
        source_id,
        "BoundaryCell32" | "BoundaryShiftCell32" | "layout_phrase"
    ) {
        CandidateOrigin::Boundary
    } else if matches!(
        source_id,
        "PhraseForecastCell32" | "L2SurfaceCompletionCell32"
    ) {
        CandidateOrigin::Completion
    } else if matches!(source_id, "L2SurfaceMotifCell32" | "L2WordAttractorCell32") {
        CandidateOrigin::L2Surface
    } else if matches!(
        source_id,
        "PhraseCell32" | "PhraseMemoryCell32" | "SemanticWordCell32"
    ) {
        CandidateOrigin::L3Context
    } else if source_id == "TechTokenCell32" {
        CandidateOrigin::Technical
    } else {
        CandidateOrigin::DeterministicTypo
    }
}

pub(super) fn gate_candidate_with_origin(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CandidateGateDecision {
    candidate_admission(original, replacement, error_class, origin)
}

fn candidate_admission(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CandidateGateDecision {
    if original == replacement {
        return CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "unchanged",
        };
    }
    let explanation = explain_candidate(original, replacement, error_class, origin);
    if explanation.blocks_apply() {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unexplained_signal_loss",
        };
    }
    if replacement_glues_separate_words_without_boundary_class(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "word_count_shrink_requires_boundary_class",
        };
    }
    if boundary_candidate_glues_short_function_tail(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_boundary_glue_short_function_tail",
        };
    }
    if boundary_candidate_eats_known_current_word(original, replacement, error_class, origin) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "moved_prefix_eats_known_current_word",
        };
    }
    if boundary_operator_changes_non_whitespace_surface(original, replacement, error_class, origin)
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "boundary_operator_changes_surface",
        };
    }
    if multi_word_candidate_only_completes_last_vowel(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_multi_word_vowel_completion",
        };
    }
    if adjacent_transposition_competes_with_single_letter_boundary(
        original,
        replacement,
        error_class,
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "single_letter_boundary_beats_transposition",
        };
    }
    if boundary_candidate_splits_known_russian_word(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_single_word_boundary_split",
        };
    }
    if boundary_candidate_splits_to_short_function_and_weak_tail(original, replacement, error_class)
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "weak_boundary_split_tail",
        };
    }
    if reflexive_suffix_candidate_requires_grammar_proof(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "reflexive_suffix_requires_grammar_proof",
        };
    }
    if known_current_word_gets_unproven_surface_drift(original, replacement, error_class, origin) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_current_word_surface_drift",
        };
    }
    if let Some(reason) =
        action_operator::verify_action_operator(original, replacement, error_class, origin)
            .apply_blocker()
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason,
        };
    }
    if surface_candidate_changes_left_context(original, replacement, origin) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "surface_left_context_apply_blocked",
        };
    }
    if l2_surface_candidate_truncates_to_stem_without_deletion_proof(original, replacement, origin)
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "l2_surface_stem_truncation_low",
        };
    }
    if let Some(decision) = structural_context_gate(original, replacement, error_class, origin) {
        return decision;
    }
    if unproven_stable_surface_shape_drift(original, replacement, error_class, origin) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unproven_stable_surface_shape_drift",
        };
    }
    if semantic_candidate_lacks_surface_authority(original, replacement, origin) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "semantic_wave_surface_authority_low",
        };
    }
    if error_class == TypingErrorClass::CompletionOnly {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "completion_is_not_autocorrect",
        };
    }
    match error_class {
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
            CandidateGateDecision {
                action: CandidateGateAction::Veto,
                reason: "protected_or_technical",
            }
        }
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
            if replacement_last_word_is_unknown_cyrillic(original, replacement) =>
        {
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "single_step_typo_still_unknown",
            }
        }
        TypingErrorClass::Unknown => CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unknown_error_class",
        },
        _ => CandidateGateDecision {
            action: CandidateGateAction::Eligible,
            reason: "class_allows_apply",
        },
    }
}

fn surface_candidate_changes_left_context(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    let source_may_only_fix_current_word = origin == CandidateOrigin::L2Surface;
    source_may_only_fix_current_word && candidate_changes_non_last_word(original, replacement)
}

fn candidate_changes_non_last_word(original: &str, replacement: &str) -> bool {
    let original_words = normalized_correction_words(original);
    let replacement_words = normalized_correction_words(replacement);
    if original_words.len() != replacement_words.len() {
        return original_words.len() > 1 || replacement_words.len() > 1;
    }
    if original_words.len() <= 1 {
        return false;
    }
    original_words[..original_words.len() - 1] != replacement_words[..replacement_words.len() - 1]
}

pub(crate) fn normalized_correction_words(text: &str) -> Vec<String> {
    crate::word_reader::normalized_text_words(text)
}

fn structural_context_gate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> Option<CandidateGateDecision> {
    if candidate_over_compresses_word(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "candidate_over_compresses_word",
        });
    }
    if candidate_drops_letter_after_one_letter_function_prefix(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "function_prefix_letter_drop",
        });
    }
    if known_phrase_part_only_grows_by_one_letter(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_phrase_part_one_letter_growth",
        });
    }
    if short_word_only_grows_initial_letter(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_initial_letter_growth",
        });
    }
    if short_word_gets_case_vowel_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_case_vowel_drift",
        });
    }
    if soft_sign_word_gets_vowel_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "soft_sign_vowel_drift",
        });
    }
    if short_word_gets_internal_consonant_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_internal_consonant_drift",
        });
    }
    if short_word_same_length_multi_edit_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_same_length_multi_edit_drift",
        });
    }
    if same_tail_single_consonant_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "same_tail_single_consonant_drift",
        });
    }
    if origin != CandidateOrigin::L3Context
        && known_russian_word_rewritten_to_different_known_word(original, replacement, error_class)
    {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_word_to_different_known_word",
        });
    }
    if short_layout_candidate_lacks_phrase_context(original, replacement, error_class, origin) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "short_layout_without_phrase_context",
        });
    }
    if short_cyrillic_word_switches_to_ascii_layout(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_cyrillic_to_ascii_layout",
        });
    }
    if short_nanda_composite_candidate_shrinks_word(original, replacement, error_class, origin) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_nanda_word_shrink",
        });
    }
    if short_nanda_candidate_inserts_internal_vowel(original, replacement, error_class, origin) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_nanda_internal_vowel_growth",
        });
    }
    if nanda_surface_candidate_outputs_unknown_word(original, replacement, error_class, origin) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "nanda_surface_unknown_word",
        });
    }
    None
}

fn reflexive_suffix_candidate_requires_grammar_proof(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class == TypingErrorClass::GrammarAgreement {
        return false;
    }
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    toggles_reflexive_soft_sign(
        &original_word.to_lowercase(),
        &replacement_word.to_lowercase(),
    )
}

fn toggles_reflexive_soft_sign(original: &str, replacement: &str) -> bool {
    let Some(stem) = original.strip_suffix("тся") else {
        return replacement
            .strip_suffix("тся")
            .is_some_and(|stem| original == format!("{stem}ться"));
    };
    replacement == format!("{stem}ться")
}

fn known_current_word_gets_unproven_surface_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower || !protected_current_surface_token(&original_lower) {
        return false;
    }

    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if original_len < 4 || replacement_len > original_len + 1 {
        return false;
    }

    damerau_levenshtein(&original_lower, &replacement_lower) <= 1
        || inserted_char_position_for_missing_letter(&original_lower, &replacement_lower).is_some()
}

fn unproven_stable_surface_shape_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }

    unproven_internal_vowel_insertion(&original_lower, &replacement_lower)
        || unproven_soft_sign_tail_insertion(&original_lower, &replacement_lower)
        || unproven_short_vowel_substitution(&original_lower, &replacement_lower)
        || unproven_tail_vowel_substitution(&original_lower, &replacement_lower)
        || unproven_inflection_tail_vowel_to_consonant(&original_lower, &replacement_lower)
}

fn unproven_internal_vowel_insertion(original: &str, replacement: &str) -> bool {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, replacement)
    else {
        return false;
    };
    if idx == 0 || !crate::russian_chars::is_russian_vowel(inserted) {
        return false;
    }
    !inserted_vowel_repairs_consonant_cluster(original, idx)
}

fn inserted_vowel_repairs_consonant_cluster(original: &str, idx: usize) -> bool {
    let chars = original.chars().collect::<Vec<_>>();
    if idx == 0 || idx >= chars.len() {
        return false;
    }
    is_russian_consonant(chars[idx - 1]) && is_russian_consonant(chars[idx])
}

fn unproven_soft_sign_tail_insertion(original: &str, replacement: &str) -> bool {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, replacement)
    else {
        return false;
    };
    inserted == 'ь' && idx + 2 >= original.chars().count()
}

fn unproven_short_vowel_substitution(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() > 6
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(original, replacement) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx > 0
        && crate::russian_chars::is_russian_vowel(**left)
        && crate::russian_chars::is_russian_vowel(**right)
}

fn unproven_tail_vowel_substitution(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() < 7
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(original, replacement) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx + 2 >= original_chars.len()
        && crate::russian_chars::is_russian_vowel(**left)
        && crate::russian_chars::is_russian_vowel(**right)
}

fn unproven_inflection_tail_vowel_to_consonant(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() < 5
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(original, replacement) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx + 3 >= original_chars.len()
        && crate::russian_chars::is_russian_vowel(**left)
        && is_russian_consonant(**right)
        && original_chars
            .last()
            .is_some_and(|ch| crate::russian_chars::is_russian_vowel(*ch))
}

fn protected_current_surface_token(lower: &str) -> bool {
    known_russian_autocorrect_token(lower)
        || crate::phrase_lexicon::is_known_russian_phrase_part(lower)
        || crate::nanda_wave::l2::l2_surface_foundation_has_authority(lower)
        || crate::russian_lexicon::is_center_backed_russian_form(lower)
        || crate::russian_lexicon::russian_dictionary().contains(lower)
        || crate::russian_lexicon::russian_short_dictionary().contains(lower)
}

fn boundary_operator_changes_non_whitespace_surface(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::BoundaryShift
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) && origin != CandidateOrigin::Boundary
    {
        return false;
    }
    original
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .ne(replacement.chars().filter(|ch| !ch.is_whitespace()))
}

fn replacement_glues_separate_words_without_boundary_class(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::BoundaryShift
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let original_words = core_word_count(original);
    let replacement_words = core_word_count(replacement);
    original_words >= 2 && replacement_words < original_words
}

fn boundary_candidate_glues_short_function_tail(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::BoundaryShift
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() != 2 || replacement_words.len() != 1 {
        return false;
    }
    let left = original_words[0];
    let right = original_words[1];
    let merged = replacement_words[0];
    if !edit_transition::same_cyrillic_token(&format!("{left}{right}"), merged) {
        return false;
    }
    let (_, right_word, _) = split_word_punctuation(right);
    let right_lower = right_word.to_lowercase();
    if matches!(right_lower.as_str(), "ся" | "сь") {
        return false;
    }
    right_word.chars().count() <= 3
        && (crate::phrase_lexicon::is_known_russian_phrase_part(&right_lower)
            || crate::russian_lexicon::is_known_russian_word_or_form(&right_lower)
            || crate::lexicon::is_common_ru_word(&right_lower))
}

fn boundary_candidate_eats_known_current_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if error_class != TypingErrorClass::BoundaryShift || origin != CandidateOrigin::Boundary {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 4
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let stripped = original_lower.chars().skip(1).collect::<String>();
    stripped == replacement_lower
}

fn multi_word_candidate_only_completes_last_vowel(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter | TypingErrorClass::CompositeTypo
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 5
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let Some(suffix) = replacement_lower.strip_prefix(&original_lower) else {
        return false;
    };
    suffix.chars().count() == 1
        && suffix.chars().next().is_some_and(|ch| {
            matches!(
                ch,
                'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я'
            )
        })
}

fn adjacent_transposition_competes_with_single_letter_boundary(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class != TypingErrorClass::AdjacentTransposition {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 4
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }
    let mut chars = original_lower.chars();
    let Some(preposition) = chars.next() else {
        return false;
    };
    if !matches!(preposition, 'в' | 'к' | 'с') {
        return false;
    }
    let tail = chars.collect::<String>();
    tail.chars().count() >= 3
        && (crate::phrase_lexicon::is_known_russian_phrase_part(&tail)
            || crate::russian_lexicon::is_known_russian_word_or_form(&tail)
            || crate::lexicon::is_common_ru_word(&tail))
}

fn boundary_candidate_splits_known_russian_word(
    original: &str,
    replacement: &str,
    _error_class: TypingErrorClass,
) -> bool {
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for original_idx in 0..original_words.len() {
        let first = replacement_words
            .get(original_idx)
            .copied()
            .unwrap_or_default();
        let second = replacement_words
            .get(original_idx + 1)
            .copied()
            .unwrap_or_default();
        let merged = format!("{first}{second}");
        if !same_known_russian_token(original_words[original_idx], &merged) {
            continue;
        }

        let before_matches = original_words[..original_idx] == replacement_words[..original_idx];
        let after_matches =
            original_words[original_idx + 1..] == replacement_words[original_idx + 2..];
        if before_matches && after_matches {
            return true;
        }
    }
    false
}

fn boundary_candidate_splits_to_short_function_and_weak_tail(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for (original_idx, original_word) in original_words.iter().enumerate() {
        let first = replacement_words
            .get(original_idx)
            .copied()
            .unwrap_or_default();
        let second = replacement_words
            .get(original_idx + 1)
            .copied()
            .unwrap_or_default();
        let merged = format!("{first}{second}");
        if !edit_transition::same_cyrillic_token(original_word, &merged) {
            continue;
        }

        let (_, first_word, _) = split_word_punctuation(first);
        let (_, second_word, _) = split_word_punctuation(second);
        let first_lower = first_word.to_lowercase();
        let second_lower = second_word.to_lowercase();
        if first_word.chars().count() == 1
            && crate::phrase_lexicon::is_one_letter_russian_function_word(&first_lower)
            && !strong_standalone_split_tail(&second_lower)
        {
            return true;
        }
    }
    false
}

pub(super) fn semantic_candidate_lacks_surface_authority(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    if origin != CandidateOrigin::L3Context {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return true;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return true;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1 {
        return false;
    }
    !crate::nanda_wave::l2::l2_center_near_surfaces(&original_lower, 24)
        .iter()
        .any(|candidate| candidate == &replacement_lower)
}

fn l2_surface_candidate_truncates_to_stem_without_deletion_proof(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    if origin != CandidateOrigin::L2Surface {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if replacement_len >= original_len || replacement_len < 4 {
        return false;
    }
    if one_deletion_reduces_to(&original_lower, &replacement_lower) {
        return false;
    }
    let prefix = crate::text_metrics::common_prefix_char_len(&original_lower, &replacement_lower);
    prefix + 1 >= replacement_len
}

fn one_deletion_reduces_to(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() != replacement_chars.len() + 1 {
        return false;
    }
    for skip in 0..original_chars.len() {
        if original_chars
            .iter()
            .enumerate()
            .filter_map(|(idx, ch)| (idx != skip).then_some(*ch))
            .eq(replacement_chars.iter().copied())
        {
            return true;
        }
    }
    false
}

fn candidate_over_compresses_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::RepeatedLetter
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_len = original_word.chars().count();
    let replacement_len = replacement_word.chars().count();
    original_len >= 6 && replacement_len + 3 <= original_len
}

fn candidate_drops_letter_after_one_letter_function_prefix(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }

    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }

    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word) || !is_cyrillic_letters_only(replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    if original_chars.len() < 4 || replacement_lower.chars().count() + 1 != original_chars.len() {
        return false;
    }
    let prefix = original_chars[0].to_string();
    if !crate::phrase_lexicon::is_one_letter_russian_function_word(&prefix) {
        return false;
    }

    let compressed = std::iter::once(original_chars[0])
        .chain(original_chars.iter().skip(2).copied())
        .collect::<String>();
    compressed == replacement_lower
}

fn known_russian_word_rewritten_to_different_known_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::BoundaryShift
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }

    known_russian_autocorrect_token(&original_lower)
        && known_russian_autocorrect_token(&replacement_lower)
}

fn known_russian_autocorrect_token(lower: &str) -> bool {
    crate::lexicon::is_common_ru_word(lower)
        || crate::lexicon::is_ru_live_protected_word(lower)
        || crate::lexicon::is_user_protected_word(lower)
        || crate::russian_lexicon::is_known_russian_word_or_form(lower)
        || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
        || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower)
        || protected_pattern_term_stem(lower)
}

fn protected_pattern_term_stem(lower: &str) -> bool {
    lower.starts_with("патерн") || lower.starts_with("паттерн")
}

fn known_phrase_part_only_grows_by_one_letter(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::AdjacentTransposition
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !is_cyrillic_letters_only(&original_word)
        || !is_cyrillic_letters_only(&replacement_word)
        || original_lower == replacement_lower
        || !crate::phrase_lexicon::is_known_russian_phrase_part(&original_lower)
    {
        return false;
    }

    inserted_char_position_for_missing_letter(&original_lower, &replacement_lower).is_some()
}

fn short_word_only_grows_initial_letter(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::AdjacentTransposition
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !is_cyrillic_letters_only(&original_word)
        || !is_cyrillic_letters_only(&replacement_word)
        || original_lower == replacement_lower
    {
        return false;
    }
    let Some((idx, _inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    idx == 0 && original_lower.chars().count() <= 6
}

fn short_word_gets_case_vowel_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 5 || !replacement_lower.starts_with(&original_lower) {
        return false;
    }
    let suffix = replacement_lower
        .strip_prefix(&original_lower)
        .unwrap_or_default();
    suffix.chars().count() == 1 && matches!(suffix, "а" | "я" | "у" | "ю" | "ы" | "и")
}

fn soft_sign_word_gets_vowel_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !original_lower.ends_with('ь') || original_lower.chars().count() > 6 {
        return false;
    }
    let original_stem = original_lower.trim_end_matches('ь');
    replacement_lower.starts_with(original_stem)
        && replacement_lower
            .chars()
            .last()
            .is_some_and(crate::russian_chars::is_russian_vowel)
}

fn short_word_gets_internal_consonant_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 6 {
        return false;
    }
    let Some((idx, inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    if crate::russian_chars::is_russian_vowel(inserted) {
        return false;
    }
    let previous_original = idx
        .checked_sub(1)
        .and_then(|previous_idx| original_lower.chars().nth(previous_idx));
    let next_original = original_lower.chars().nth(idx);
    if Some(inserted) == previous_original || Some(inserted) == next_original {
        return false;
    }
    !(inserted == 'ч' && matches!(next_original, Some('ш' | 'щ')))
}

fn short_word_same_length_multi_edit_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    original_len <= 6
        && original_len == replacement_len
        && damerau_levenshtein(&original_lower, &replacement_lower) >= 2
}

fn same_tail_single_consonant_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    let replacement_chars = replacement_lower.chars().collect::<Vec<_>>();
    if original_chars.len() < 6
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(&original_lower, &replacement_lower) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx > 1
        && *idx + 2 < original_chars.len()
        && is_russian_consonant(**left)
        && is_russian_consonant(**right)
        && original_chars[original_chars.len() - 2..]
            == replacement_chars[replacement_chars.len() - 2..]
}

fn is_russian_consonant(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
        && !crate::russian_chars::is_russian_vowel(ch)
        && !matches!(ch, 'ь' | 'ъ')
}

fn short_layout_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(
        origin,
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
    ) || !matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::MixedScript
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word.chars().count() != 1 || replacement_word.chars().count() != 1 {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let previous_words = original_core
        .split_whitespace()
        .take(original_core.split_whitespace().count().saturating_sub(1))
        .collect::<Vec<_>>();
    let has_cyrillic_context = previous_words.iter().any(|word| has_cyrillic(word));
    let has_ascii_context = previous_words
        .iter()
        .any(|word| word.chars().any(|ch| ch.is_ascii_alphabetic()));

    has_ascii_context && !has_cyrillic_context
}

fn short_cyrillic_word_switches_to_ascii_layout(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class != TypingErrorClass::WrongLayout {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    original_word.chars().count() <= 2
        && original_word
            .chars()
            .any(|ch| matches!(ch, 'а'..='я' | 'ё' | 'А'..='Я' | 'Ё'))
        && replacement_word
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '`'))
}

fn short_nanda_composite_candidate_shrinks_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_len = original_word.chars().count();
    let replacement_len = replacement_word.chars().count();
    original_len <= 4 && replacement_len < original_len
}

fn nanda_surface_candidate_outputs_unknown_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word == replacement_word || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let replacement_lower = replacement_word.to_lowercase();
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
        && !crate::phrase_lexicon::is_known_russian_phrase_part(&replacement_lower)
}

fn short_nanda_candidate_inserts_internal_vowel(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 6
        || damerau_levenshtein(&original_lower, &replacement_lower) != 1
    {
        return false;
    }
    let Some((idx, inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    idx > 0 && crate::russian_chars::is_russian_vowel(inserted)
}

fn same_known_russian_token(original: &str, candidate: &str) -> bool {
    let (_, original_word, _) = split_word_punctuation(original);
    let (_, candidate_word, _) = split_word_punctuation(candidate);
    if original_word.is_empty()
        || candidate_word.is_empty()
        || !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(candidate_word)
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    original_lower == candidate_word.to_lowercase()
        && (crate::nanda_wave::l2::l2_surface_foundation_contains(&original_lower)
            || crate::russian_lexicon::is_reference_backed_russian_form(&original_lower)
            || crate::lexicon::is_common_ru_word(&original_lower)
            || crate::lexicon::is_ru_technical_loanword(&original_lower))
}

fn strong_standalone_split_tail(lower: &str) -> bool {
    lower.chars().count() >= 4
        && (crate::lexicon::is_common_ru_word(lower)
            || crate::russian_lexicon::russian_dictionary().contains(lower)
            || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
            || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower))
}

fn core_word_count(text: &str) -> usize {
    let (_, core, _) = split_edge_whitespace(text);
    core.split_whitespace().count()
}

fn replacement_last_word_is_unknown_cyrillic(original: &str, replacement: &str) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word == replacement_word || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if repeated_deletion_has_surface_support(&original_lower, &replacement_lower) {
        return false;
    }
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
}

pub(super) fn repeated_deletion_has_surface_support(
    original_lower: &str,
    replacement_lower: &str,
) -> bool {
    if !repeated_run_deletion_candidates(original_lower)
        .into_iter()
        .any(|candidate| candidate == replacement_lower)
    {
        return false;
    }
    crate::russian_lexicon::is_known_russian_word_or_form(replacement_lower)
        || crate::lexicon::is_common_ru_word(replacement_lower)
        || short_final_repeated_vowel_delete_has_surface_support(original_lower, replacement_lower)
}

fn short_final_repeated_vowel_delete_has_surface_support(
    original_lower: &str,
    replacement_lower: &str,
) -> bool {
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    let replacement_chars = replacement_lower.chars().collect::<Vec<_>>();
    if original_chars.len() > 5 || original_chars.len() != replacement_chars.len() + 1 {
        return false;
    }
    let Some(&last) = original_chars.last() else {
        return false;
    };
    if last != 'и'
        || !crate::russian_chars::is_russian_vowel(last)
        || original_chars
            .get(original_chars.len().saturating_sub(2))
            .copied()
            != Some(last)
    {
        return false;
    }
    replacement_chars.as_slice() == &original_chars[..original_chars.len() - 1]
        && crate::russian_typo_scoring::ngram_allows_ru_candidate(
            replacement_lower,
            original_lower,
            REPEATED_DELETE_SURFACE_MARGIN,
        )
}

pub(super) fn should_prefer_composite_after_repeated_repair(
    original: &str,
    single_step: &str,
    composite: &str,
) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(single_word) = last_text_word(single_step) else {
        return false;
    };
    let Some(composite_word) = last_text_word(composite) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let single_lower = single_word.to_lowercase();
    let composite_lower = composite_word.to_lowercase();
    if single_lower == composite_lower || !is_cyrillic_letters_only(&composite_word) {
        return false;
    }
    if single_word.chars().count() < original_word.chars().count()
        && composite_word.chars().count() > original_word.chars().count()
        && damerau_levenshtein(&original_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
    {
        return true;
    }
    repeated_run_deletion_candidates(&original_lower)
        .into_iter()
        .any(|candidate| candidate == single_lower)
        && composite_word.chars().count() > original_word.chars().count()
        && damerau_levenshtein(&single_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
}
