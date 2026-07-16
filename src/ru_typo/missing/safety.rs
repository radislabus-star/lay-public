fn safe_missing_letter_candidates_impl(lower: &str) -> impl Iterator<Item = String> + '_ {
    generate_missing_letter_candidates(lower)
        .filter(move |candidate| is_safe_missing_letter_candidate(lower, candidate))
}

fn is_safe_missing_letter_candidate(lower: &str, candidate: &str) -> bool {
    if let Some((idx, inserted)) = inserted_char_position_for_missing_letter(lower, candidate) {
        if is_risky_vowel_insert_into_verb_tail(lower, inserted) {
            return false;
        }
        if is_risky_consonant_insert_before_final_verb_tail(lower, idx, inserted) {
            return false;
        }
        if idx == lower.chars().count() {
            if lower.ends_with("ств") {
                return false;
            }
            return is_russian_vowel(inserted)
                && lower
                    .chars()
                    .last()
                    .is_some_and(|last| !is_russian_vowel(last));
        }
    }
    if let Some(inserted) = candidate.strip_suffix(lower) {
        if inserted == "о" && is_known_russian_word_or_form(candidate) {
            return true;
        }
        return inserted.chars().count() != 1 || lower.chars().next().is_some_and(is_russian_vowel);
    }

    true
}

fn is_risky_consonant_insert_before_final_verb_tail(
    lower: &str,
    idx: usize,
    inserted: char,
) -> bool {
    !is_russian_vowel(inserted)
        && ["ти", "ть"].iter().any(|tail| {
            lower.ends_with(tail)
                && idx >= lower.chars().count().saturating_sub(tail.chars().count())
        })
}

fn is_risky_vowel_insert_into_verb_tail(lower: &str, inserted: char) -> bool {
    is_russian_vowel(inserted)
        && [
            "аешь", "яешь", "еешь", "оешь", "уешь", "ешь", "ишь", "еет", "ает", "яет", "ует",
        ]
        .iter()
        .any(|tail| lower.ends_with(tail))
}
