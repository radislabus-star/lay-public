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
