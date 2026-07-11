use super::*;

#[test]
fn missing_letter_repairs_known_weak_form_to_common_word() {
    let candidates = safe_missing_letter_candidates("недоказно").collect::<Vec<_>>();
    assert!(
        candidates.iter().any(|candidate| candidate == "недоказано"),
        "safe missing-letter candidates: {candidates:?}"
    );
    assert!(
        has_common_missing_letter_candidate("недоказно"),
        "expected common missing-letter candidate"
    );
    assert_eq!(
        correct_missing_letter("недоказно").as_deref(),
        Some("недоказано")
    );
}

#[test]
fn missing_letter_rejects_consonant_insert_before_final_ti_tail() {
    assert!(
        !safe_missing_letter_candidates("зачати").any(|candidate| candidate == "зачасти"),
        "final -ти consonant insertion must not be an autocorrect authority"
    );
    assert_eq!(correct_missing_letter("зачати"), None);
}

#[test]
fn missing_letter_uses_center_backed_inflected_surfaces() {
    for (dirty, expected) in [("дейстия", "действия"), ("лушее", "лучшее")]
    {
        assert!(
            !crate::nanda_wave::l2::l2_surface_foundation_has_authority(dirty)
                && !crate::russian_lexicon::is_center_backed_russian_form(dirty),
            "dirty surface unexpectedly has trusted field authority: {dirty}"
        );
        let candidates = safe_missing_letter_candidates(dirty).collect::<Vec<_>>();
        assert!(
            candidates.iter().any(|candidate| candidate == expected),
            "operator did not produce {expected}: {candidates:?}"
        );
        let ranked = candidates
            .iter()
            .filter(|candidate| is_known_russian_word_or_form(candidate))
            .map(|candidate| {
                (
                    candidate,
                    crate::ngram::ru_candidate_margin(candidate, dirty)
                        + missing_letter_candidate_bonus(dirty, candidate),
                )
            })
            .collect::<Vec<_>>();
        let direct = best_ranked_dictionary_candidate(
            dirty,
            candidates.clone(),
            NGRAM_DICT_MISSING_LETTER_MARGIN,
            0.40,
        );
        assert_eq!(
            correct_missing_letter(dirty).as_deref(),
            Some(expected),
            "known candidates: {ranked:?}; direct={direct:?}; past={}; prefix={}; exists={}; vowel_exists={}",
            looks_like_plausible_russian_past_tense(dirty),
            looks_like_prefix_plus_known_russian_word(dirty),
            missing_letter_candidate_exists(dirty, dirty),
            vowel_nonverb_missing_letter_candidate_exists(dirty, dirty),
            // keep authority diagnostics near the regression assertion
        );
    }
}
