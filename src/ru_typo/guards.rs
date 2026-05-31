use crate::keyboard::is_cyrillic_letter;
use crate::phrase_lexicon::{is_known_russian_phrase_part, is_one_letter_russian_function_word};
use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::{
    is_known_russian_word_or_form, looks_like_russian_adjective_lemma, russian_dictionary,
};
use crate::word_reader::is_cyrillic_word;

pub(super) fn unknown_cyrillic_lower(word: &str, min_chars: usize) -> Option<String> {
    if word.chars().count() < min_chars || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    (!is_known_russian_word_or_form(&lower)).then_some(lower)
}

pub(super) fn correct_invalid_adjective_tail(original: &str, lower: &str) -> Option<String> {
    let stem = lower
        .strip_suffix('ы')
        .or_else(|| lower.strip_suffix('и'))?;
    if !looks_like_russian_adjective_lemma(stem) || !is_known_russian_word_or_form(stem) {
        return None;
    }
    Some(crate::text_case::apply_word_case(original, stem))
}

pub(super) fn looks_like_present_or_reflexive_verb(word: &str) -> bool {
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

pub(super) fn looks_like_known_word_plus_one_letter_function_suffix(candidate: &str) -> bool {
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

pub(super) fn looks_like_prefix_plus_known_russian_word(lower: &str) -> bool {
    let chars: Vec<char> = lower.chars().collect();
    (1..=2).any(|prefix_len| {
        chars.len() > prefix_len + 3
            && is_known_russian_word_or_form(&chars[prefix_len..].iter().collect::<String>())
    })
}

pub(super) fn looks_like_plausible_russian_past_tense(word: &str) -> bool {
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
