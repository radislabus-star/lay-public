use super::token::{
    is_known_word, is_layout_garbage_token, is_natural_hyphenated_token, trim_token,
};
use crate::lexicon::is_ru_short_function_word;
use crate::ngram;
use crate::word_recognizer::{
    is_ascii_technical_or_brand_token, is_mixed_cyrillic_ascii_alpha_token,
};

pub(super) fn language_score(text: &str) -> f64 {
    let mut total = 0.0;
    let mut count = 0usize;
    for token in text.split_whitespace() {
        let token = trim_token(token);
        if token.is_empty() {
            continue;
        }
        total += token_language_score(token);
        count += 1;
    }
    if count == 0 {
        -20.0
    } else {
        total / count as f64
    }
}

fn token_language_score(token: &str) -> f64 {
    let lower = token.to_lowercase();
    if is_ru_short_function_word(&lower) {
        return -5.5;
    }
    if is_mixed_cyrillic_ascii_alpha_token(token) {
        return -22.0;
    }
    if is_layout_garbage_token(token) {
        return -18.0;
    }
    if is_ascii_technical_or_brand_token(token) {
        return -5.0;
    }

    let alpha_count = token.chars().filter(|ch| ch.is_alphabetic()).count().max(1) as f64;
    let ru = ngram::ru_score(token);
    let en = ngram::en_score(token);
    let ru_norm = if ru.is_finite() {
        ru / alpha_count
    } else {
        -20.0
    };
    let en_norm = if en.is_finite() {
        en / alpha_count
    } else {
        -20.0
    };
    ru_norm.max(en_norm) + lexical_bonus(token)
}

fn lexical_bonus(token: &str) -> f64 {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return 0.0;
    }
    if is_known_word(token) {
        return 1.15;
    }
    if token.contains('-')
        && token
            .split('-')
            .filter(|part| !part.is_empty())
            .all(is_known_word)
    {
        return 0.90;
    }
    if is_natural_hyphenated_token(token) {
        return 0.20;
    }
    if token.chars().any(|ch| ch.is_alphabetic()) {
        return -0.55;
    }
    0.0
}
