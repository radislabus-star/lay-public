//! Russian typo correction rules for typing assist.
//!
//! This module owns local word-level typo candidates only: missing letters,
//! extra letters, adjacent swaps, repeated letters, vowel confusion and nearby
//! keyboard substitutions. It does not know about daemon runtime or text output.

use crate::keyboard::is_cyrillic_letter;
use crate::phrase_lexicon::{
    is_known_russian_phrase_part, is_one_letter_russian_function_word,
    is_short_russian_function_word, looks_like_short_function_word_glued_to_known_word,
};
use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::{
    is_known_russian_word_or_form, looks_like_russian_adjective_lemma, russian_dictionary,
};
use crate::russian_typo_candidates::{
    generate_extra_letter_candidates, generate_hard_sign_candidates,
    generate_missing_letter_candidates, generate_vowel_confusion_candidates,
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates, RU_ALPHABET,
};
use crate::russian_typo_scoring::{
    best_ranked_dictionary_candidate, best_unique_known_ngram_candidate,
    missing_letter_candidate_bonus, ngram_allows_ru_candidate,
};
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

const NGRAM_TYPO_REJECT_MARGIN: f64 = 0.25;
const NGRAM_TRANSPOSE_MARGIN: f64 = -8.0;
const NGRAM_DICT_MISSING_LETTER_MARGIN: f64 = -8.0;
const NGRAM_EXTRA_LETTER_MARGIN: f64 = 0.75;
const NGRAM_VOWEL_CONFUSION_MARGIN: f64 = -1.0;
const NGRAM_VERB_ENDING_MARGIN: f64 = -8.0;
const NGRAM_HARD_SIGN_MARGIN: f64 = 1.0;

pub(crate) fn correct_cyrillic_word_case(word: &str) -> Option<String> {
    if word.chars().count() < 2 || !is_cyrillic_word(word) {
        return None;
    }
    if word
        .chars()
        .all(|ch| !ch.is_alphabetic() || !ch.is_uppercase())
        || word
            .chars()
            .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        return None;
    }

    let lower = word.to_lowercase();
    if !is_known_russian_word_or_form(&lower) {
        return None;
    }

    let normalized = apply_word_case(word, &lower);
    (normalized != word).then_some(normalized)
}

pub(crate) fn correct_hard_sign_typo(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    best_unique_known_ngram_candidate(
        word,
        generate_hard_sign_candidates(&lower),
        NGRAM_HARD_SIGN_MARGIN,
    )
}

pub(crate) fn correct_adjacent_transposition(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if missing_letter_candidate_exists(word, &lower) {
        return None;
    }
    if extra_letter_candidate_exists(&lower) {
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
        if !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if looks_like_known_word_plus_one_letter_function_suffix(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TRANSPOSE_MARGIN) {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

pub(crate) fn correct_repeated_letter(word: &str) -> Option<String> {
    if !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if let Some(candidate) = correct_short_repeated_function_word(word, &lower) {
        return Some(candidate);
    }
    if word.chars().count() < 5 {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    let mut idx = 0;
    while idx < chars.len() {
        let mut end = idx + 1;
        while end < chars.len() && chars[end] == chars[idx] {
            end += 1;
        }

        if end - idx > 1 {
            for keep in 1..end - idx {
                let mut candidate = Vec::with_capacity(chars.len() - (end - idx - keep));
                candidate.extend_from_slice(&chars[..idx]);
                candidate.extend(std::iter::repeat(chars[idx]).take(keep));
                candidate.extend_from_slice(&chars[end..]);
                let candidate: String = candidate.into_iter().collect();
                if !is_known_russian_word_or_form(&candidate) {
                    continue;
                }
                if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TYPO_REJECT_MARGIN) {
                    continue;
                }
                if found.is_some() {
                    return None;
                }
                found = Some(candidate);
            }
        }

        idx = end;
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn correct_short_repeated_function_word(original: &str, lower: &str) -> Option<String> {
    if !(3..=4).contains(&lower.chars().count()) {
        return None;
    }

    let mut found: Option<String> = None;
    for candidate in repeated_run_deletion_candidates(lower) {
        if candidate.chars().count() < 2 {
            continue;
        }
        if !is_short_russian_function_word(&candidate) {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(original, &candidate))
}

pub(crate) fn correct_single_letter_substitution(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if missing_letter_candidate_exists(word, &lower) {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    for idx in 0..chars.len() {
        // First-letter substitutions are too ambiguous for automatic correction:
        // slang, names and dialect forms often differ from dictionary words only there.
        if idx == 0 {
            continue;
        }
        for replacement in RU_ALPHABET {
            if replacement == chars[idx] {
                continue;
            }
            if !are_ru_keyboard_neighbors(chars[idx], replacement) {
                continue;
            }

            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            let candidate: String = candidate.into_iter().collect();
            if !is_known_russian_word_or_form(&candidate) {
                continue;
            }
            if looks_like_known_word_plus_one_letter_function_suffix(&candidate) {
                continue;
            }
            if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TYPO_REJECT_MARGIN) {
                continue;
            }

            if found.is_some() {
                return None;
            }
            found = Some(candidate);
        }
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

pub fn correct_extra_letters(word: &str) -> Option<String> {
    if word.chars().count() < 6 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if lower.ends_with("тся") {
        return None;
    }
    if looks_like_short_function_word_glued_to_known_word(&lower) {
        return None;
    }
    if let Some(candidate) = correct_invalid_adjective_tail(word, &lower) {
        return Some(candidate);
    }
    if missing_letter_candidate_exists(word, &lower) {
        return None;
    }

    best_unique_known_ngram_candidate(
        word,
        generate_extra_letter_candidates(&lower),
        NGRAM_EXTRA_LETTER_MARGIN,
    )
}

fn correct_invalid_adjective_tail(original: &str, lower: &str) -> Option<String> {
    let stem = lower
        .strip_suffix('ы')
        .or_else(|| lower.strip_suffix('и'))?;
    if !looks_like_russian_adjective_lemma(stem) || !is_known_russian_word_or_form(stem) {
        return None;
    }
    Some(apply_word_case(original, stem))
}

pub(crate) fn correct_vowel_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if looks_like_plausible_russian_past_tense(&lower) {
        return None;
    }

    best_unique_known_ngram_candidate(
        word,
        generate_vowel_confusion_candidates(&lower),
        NGRAM_VOWEL_CONFUSION_MARGIN,
    )
}

pub(crate) fn correct_verb_ending_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    if let Some(stem) = lower.strip_suffix("тся") {
        if stem.chars().count() >= 3 {
            let candidate = format!("{stem}ться");
            if is_known_russian_word_or_form(&candidate)
                && ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN)
            {
                return Some(apply_word_case(word, &candidate));
            }
        }
    }

    for (from, to) in [("ешь", "ишь"), ("ет", "ит")] {
        let Some(stem) = lower.strip_suffix(from) else {
            continue;
        };
        if stem.chars().count() < 3 {
            continue;
        }
        let candidate = format!("{stem}{to}");
        if !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN) {
            continue;
        }
        return Some(apply_word_case(word, &candidate));
    }

    None
}

pub fn correct_missing_letter(word: &str) -> Option<String> {
    if word.chars().count() < 6 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if looks_like_plausible_russian_past_tense(&lower) {
        return None;
    }
    if looks_like_prefix_plus_known_russian_word(&lower)
        && !vowel_nonverb_missing_letter_candidate_exists(word, &lower)
    {
        return None;
    }

    best_ranked_dictionary_candidate(
        word,
        safe_missing_letter_candidates(&lower),
        NGRAM_DICT_MISSING_LETTER_MARGIN,
        0.40,
    )
}

fn missing_letter_candidate_exists(word: &str, lower: &str) -> bool {
    let original_lower = word.to_lowercase();
    safe_missing_letter_candidates(lower).any(|candidate| {
        candidate != original_lower
            && is_known_russian_word_or_form(&candidate)
            && crate::ngram::ru_candidate_margin(&candidate, &original_lower)
                + missing_letter_candidate_bonus(&original_lower, &candidate)
                >= NGRAM_DICT_MISSING_LETTER_MARGIN
    })
}

fn vowel_nonverb_missing_letter_candidate_exists(word: &str, lower: &str) -> bool {
    let original_lower = word.to_lowercase();
    safe_missing_letter_candidates(lower).any(|candidate| {
        let Some((_, inserted)) = inserted_char_position_for_missing_letter(lower, &candidate)
        else {
            return false;
        };
        is_russian_vowel(inserted)
            && !looks_like_present_or_reflexive_verb(&candidate)
            && candidate != original_lower
            && is_known_russian_word_or_form(&candidate)
            && crate::ngram::ru_candidate_margin(&candidate, &original_lower)
                + missing_letter_candidate_bonus(&original_lower, &candidate)
                >= NGRAM_DICT_MISSING_LETTER_MARGIN
    })
}

fn looks_like_present_or_reflexive_verb(word: &str) -> bool {
    [
        "ается",
        "яется",
        "уется",
        "ется",
        "ются",
        "ешь",
        "ишь",
        "аете",
        "яете",
        "ите",
        "ает",
        "яет",
        "ует",
        "ают",
        "яют",
        "ит",
        "ет",
    ]
    .iter()
    .any(|ending| word.ends_with(ending))
}

fn extra_letter_candidate_exists(lower: &str) -> bool {
    generate_extra_letter_candidates(lower)
        .into_iter()
        .any(|candidate| {
            candidate != lower
                && is_known_russian_word_or_form(&candidate)
                && crate::ngram::ru_candidate_margin(&candidate, lower) >= NGRAM_EXTRA_LETTER_MARGIN
        })
}

fn looks_like_known_word_plus_one_letter_function_suffix(candidate: &str) -> bool {
    if russian_dictionary().contains(candidate) {
        return false;
    }
    for split_at in candidate.char_indices().skip(1).map(|(idx, _)| idx) {
        let (left, right) = candidate.split_at(split_at);
        if right.chars().count() != 1 {
            continue;
        }
        if left.chars().count() < 4 {
            continue;
        }
        if is_known_russian_phrase_part(left) && is_one_letter_russian_function_word(right) {
            return true;
        }
    }
    false
}

pub(crate) fn safe_missing_letter_candidates(lower: &str) -> impl Iterator<Item = String> + '_ {
    generate_missing_letter_candidates(lower)
        .filter(move |candidate| is_safe_missing_letter_candidate(lower, candidate))
}

fn is_safe_missing_letter_candidate(lower: &str, candidate: &str) -> bool {
    if let Some((idx, inserted)) = inserted_char_position_for_missing_letter(lower, candidate) {
        if idx == lower.chars().count() {
            return is_russian_vowel(inserted)
                && lower
                    .chars()
                    .last()
                    .is_some_and(|last| !is_russian_vowel(last));
        }
    }
    if let Some(inserted) = candidate.strip_suffix(lower) {
        return inserted.chars().count() != 1 || lower.chars().next().is_some_and(is_russian_vowel);
    }

    true
}

pub(crate) fn has_plausible_russian_typo_candidate(lower: &str) -> bool {
    if lower.chars().count() < 5 || !is_cyrillic_word(lower) || is_known_russian_word_or_form(lower)
    {
        return false;
    }

    correct_verb_ending_confusion(lower).is_some()
        || correct_hard_sign_typo(lower).is_some()
        || correct_repeated_letter(lower).is_some()
        || correct_adjacent_transposition(lower).is_some()
        || correct_missing_letter(lower).is_some()
        || correct_single_letter_substitution(lower).is_some()
        || correct_vowel_confusion(lower).is_some()
        || correct_extra_letters(lower).is_some()
        || has_generated_russian_typo_candidate(lower)
}

fn has_generated_russian_typo_candidate(lower: &str) -> bool {
    safe_missing_letter_candidates(lower).any(|candidate| {
        candidate != lower
            && is_known_russian_word_or_form(&candidate)
            && crate::ngram::ru_candidate_margin(&candidate, lower)
                + missing_letter_candidate_bonus(lower, &candidate)
                >= NGRAM_DICT_MISSING_LETTER_MARGIN
    }) || generate_vowel_confusion_candidates(lower)
        .into_iter()
        .any(|candidate| candidate != lower && is_known_russian_word_or_form(&candidate))
        || generate_extra_letter_candidates(lower)
            .into_iter()
            .any(|candidate| {
                candidate != lower
                    && is_known_russian_word_or_form(&candidate)
                    && crate::ngram::ru_candidate_margin(&candidate, lower)
                        >= NGRAM_EXTRA_LETTER_MARGIN
            })
}

fn looks_like_prefix_plus_known_russian_word(lower: &str) -> bool {
    let chars: Vec<char> = lower.chars().collect();
    (1..=2).any(|prefix_len| {
        chars.len() > prefix_len + 3
            && is_known_russian_word_or_form(&chars[prefix_len..].iter().collect::<String>())
    })
}

pub fn are_ru_keyboard_neighbors(a: char, b: char) -> bool {
    let Some((row_a, col_a)) = ru_keyboard_position(a) else {
        return false;
    };
    let Some((row_b, col_b)) = ru_keyboard_position(b) else {
        return false;
    };

    row_a == row_b && col_a.abs_diff(col_b) <= 1
}

fn ru_keyboard_position(ch: char) -> Option<(usize, usize)> {
    const ROWS: [&str; 3] = ["йцукенгшщзхъ", "фывапролджэ", "ячсмитьбю"];
    ROWS.iter()
        .enumerate()
        .find_map(|(row, keys)| keys.chars().position(|key| key == ch).map(|col| (row, col)))
}

fn looks_like_plausible_russian_past_tense(word: &str) -> bool {
    const ENDINGS: &[&str] = &[
        "илась",
        "ились",
        "илось",
        "алась",
        "ались",
        "алось",
        "ила",
        "или",
        "ило",
        "ала",
        "али",
        "ало",
        "ела",
        "ели",
        "ело",
        "ил",
        "ал",
        "ел",
    ];

    ENDINGS.iter().any(|ending| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        let stem_len = stem.chars().count();
        stem_len >= 2
            && stem.chars().any(is_russian_vowel)
            && stem
                .chars()
                .any(|ch| is_cyrillic_letter(ch) && !is_russian_vowel(ch))
    })
}
