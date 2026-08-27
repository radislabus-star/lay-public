// Surface authority, lexical support, and repeated-repair predicates.

pub(crate) fn semantic_candidate_lacks_surface_authority(
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
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
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

fn known_russian_word_rewritten_to_different_known_word_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return known_russian_word_rewritten_to_different_known_word(
            original,
            replacement,
            error_class,
        );
    }
    facts.assert_pair(original, replacement);
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::BoundaryShift
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if !original_word.is_cyrillic_letters_only() || !replacement_word.is_cyrillic_letters_only() {
        return false;
    }
    if original_word.lower() == replacement_word.lower() {
        return false;
    }
    original_word.is_known_russian() && replacement_word.is_known_russian()
}

fn known_russian_autocorrect_token(lower: &str) -> bool {
    crate::lexicon::is_common_ru_word(lower)
        || crate::lexicon::is_ru_live_protected_word(lower)
        || crate::lexicon::is_user_protected_word(lower)
        || crate::nanda_wave::l2::l2_surface_foundation_contains(lower)
        || crate::russian_lexicon::is_reference_backed_russian_form(lower)
        || crate::russian_lexicon::is_reference_known_russian_word_or_form(lower)
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
    if verified_surface_to_lexical_center_repair(&original_lower, &replacement_lower, error_class) {
        return false;
    }
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
    let immediate_entity_context = previous_words.last().is_some_and(|word| {
        crate::word_recognizer::is_ascii_titlecase_token(word)
            || crate::word_recognizer::is_ascii_technical_or_brand_token(word)
    });

    has_ascii_context && !has_cyrillic_context && !immediate_entity_context
}

fn short_cyrillic_word_switches_to_ascii_layout(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
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
    let exact_known_layout_center = origin == CandidateOrigin::Layout
        && crate::dict::convert(&original_word, crate::dict::Direction::Ru2Us)
            .eq_ignore_ascii_case(&replacement_word)
        && crate::layout_autoswitch::is_known_english_layout_autoswitch_word(
            &replacement_word.to_ascii_lowercase(),
        );
    original_word.chars().count() <= 3
        && original_word
            .chars()
            .any(|ch| matches!(ch, 'а'..='я' | 'ё' | 'А'..='Я' | 'Ё'))
        && replacement_word
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '`'))
        && !exact_known_layout_center
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

fn nanda_surface_candidate_outputs_unknown_word_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return nanda_surface_candidate_outputs_unknown_word(
            original,
            replacement,
            error_class,
            origin,
        );
    }
    facts.assert_pair(original, replacement);
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if original_word.word() == replacement_word.word()
        || !replacement_word.is_cyrillic_letters_only()
    {
        return false;
    }
    !crate::russian_lexicon::is_known_russian_word_or_form(replacement_word.lower())
        && !crate::lexicon::is_common_ru_word(replacement_word.lower())
        && !crate::phrase_lexicon::is_known_russian_phrase_part(replacement_word.lower())
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
    let len = lower.chars().count();
    (len >= 3 && crate::lexicon::is_common_ru_word(lower))
        || (len >= 4
            && (crate::russian_lexicon::russian_dictionary().contains(lower)
                || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
                || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower)))
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

pub(crate) fn repeated_deletion_has_surface_support(
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

pub(crate) fn should_prefer_composite_after_repeated_repair(
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
