use super::token::{
    is_known_word, is_layout_garbage_token, is_natural_hyphenated_token, trim_token,
};
use crate::lexicon::is_ru_short_function_word;
use crate::ngram;
use crate::word_recognizer::{
    is_ascii_technical_or_brand_token, is_mixed_cyrillic_ascii_alpha_token,
};

const EMPTY_TEXT_SCORE: f64 = -20.0;
const SHORT_RU_FUNCTION_WORD_SCORE: f64 = -5.5;
const MIXED_SCRIPT_TOKEN_SCORE: f64 = -22.0;
const LAYOUT_GARBAGE_TOKEN_SCORE: f64 = -18.0;
const ASCII_TECHNICAL_TOKEN_SCORE: f64 = -5.0;
const UNKNOWN_LANGUAGE_SCORE: f64 = -20.0;
const KNOWN_WORD_BONUS: f64 = 1.15;
const KNOWN_HYPHENATED_WORD_BONUS: f64 = 0.90;
const NATURAL_HYPHENATED_TOKEN_BONUS: f64 = 0.20;
const UNKNOWN_ALPHABETIC_TOKEN_PENALTY: f64 = -0.55;

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
        EMPTY_TEXT_SCORE
    } else {
        total / count as f64
    }
}

fn token_language_score(token: &str) -> f64 {
    let lower = token.to_lowercase();
    if is_ru_short_function_word(&lower) {
        return SHORT_RU_FUNCTION_WORD_SCORE;
    }
    if is_mixed_cyrillic_ascii_alpha_token(token) {
        return MIXED_SCRIPT_TOKEN_SCORE;
    }
    if is_layout_garbage_token(token) {
        return LAYOUT_GARBAGE_TOKEN_SCORE;
    }
    if is_ascii_technical_or_brand_token(token) {
        return ASCII_TECHNICAL_TOKEN_SCORE;
    }

    let alpha_count = token.chars().filter(|ch| ch.is_alphabetic()).count().max(1) as f64;
    let ru = ngram::ru_score(token);
    let en = ngram::en_score(token);
    let ru_norm = if ru.is_finite() {
        ru / alpha_count
    } else {
        UNKNOWN_LANGUAGE_SCORE
    };
    let en_norm = if en.is_finite() {
        en / alpha_count
    } else {
        UNKNOWN_LANGUAGE_SCORE
    };
    ru_norm.max(en_norm) + lexical_bonus(token)
}

fn lexical_bonus(token: &str) -> f64 {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return 0.0;
    }
    if is_known_word(token) {
        return KNOWN_WORD_BONUS;
    }
    if token.contains('-')
        && token
            .split('-')
            .filter(|part| !part.is_empty())
            .all(is_known_word)
    {
        return KNOWN_HYPHENATED_WORD_BONUS;
    }
    if is_natural_hyphenated_token(token) {
        return NATURAL_HYPHENATED_TOKEN_BONUS;
    }
    if token.chars().any(|ch| ch.is_alphabetic()) {
        return UNKNOWN_ALPHABETIC_TOKEN_PENALTY;
    }
    0.0
}
