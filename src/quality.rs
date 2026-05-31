//! Legacy/auxiliary plausibility score for converted text.
//!
//! This module is intentionally lightweight and has no external dictionary
//! dependency. The current public CLI path is deterministic; the daemon's smart
//! path uses dictionaries/ngram/token rules first. These helpers remain useful
//! for tests and conservative fallback scoring.
//!
//! The score is only a plausibility hint, not a production guarantee.

use crate::russian_chars::is_russian_vowel;

const EN_VOWELS: &str = "aeiouAEIOU";

/// Доля «правдоподобных» слов в тексте (0..1).
pub fn score(text: &str, lang: &str) -> f32 {
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return 1.0;
    }

    let plausible = words.iter().filter(|w| is_plausible_word(w, lang)).count();
    plausible as f32 / words.len() as f32
}

fn is_plausible_word(word: &str, lang: &str) -> bool {
    let len = word.chars().count();

    // Слова длиной 1-2 символа всегда считаем плауcибельными
    if len <= 2 {
        return true;
    }

    let vowel_count = word.chars().filter(|ch| is_lang_vowel(*ch, lang)).count();

    // 1. Хотя бы одна гласная (русский: 1 на 6 символов, английский: 1 на 5)
    let min_vowels = match lang {
        "ru" => (len as f32 / 6.0).ceil() as usize,
        _ => (len as f32 / 5.0).ceil() as usize,
    }
    .max(1);
    if vowel_count < min_vowels {
        return false;
    }

    // 2. Нет 4+ согласных подряд
    let mut consonant_streak = 0;
    for c in word.chars() {
        if c.is_alphabetic() && !is_lang_vowel(c, lang) {
            consonant_streak += 1;
            if consonant_streak >= 4 {
                return false;
            }
        } else {
            consonant_streak = 0;
        }
    }

    true
}

fn is_lang_vowel(ch: char, lang: &str) -> bool {
    match lang {
        "ru" => is_russian_vowel(ch),
        _ => EN_VOWELS.contains(ch),
    }
}

#[cfg(test)]
#[path = "quality_tests.rs"]
mod tests;
