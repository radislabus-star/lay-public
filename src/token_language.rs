//! Token-level language recognition for layout arbiters.
//!
//! This is a small lexical layer: normalize a token, decide whether it is known
//! Russian/English, and evaluate whole-token sequences. It does not correct or
//! emit text.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::lexicon::{
    is_ru_one_letter_function_word, is_ru_single_letter_pronoun, EN_HUNSPELL, EN_WORDS,
};
use crate::russian_lexicon::{is_known_russian_word_or_form, russian_tiny_dictionary};
use crate::word_reader::split_ws_segments;
use crate::word_recognizer::is_protected_ascii_token;

#[derive(Clone, Copy)]
pub(crate) enum Lang {
    Ru,
    En,
}

pub(crate) fn warm_up() {
    crate::russian_lexicon::warm_up();
    let _ = en_dictionary().len();
}

pub(crate) fn is_known_ru_token(token: &str) -> bool {
    let Some(word) = normalized_token_core(token, Lang::Ru) else {
        return false;
    };
    is_known_ru_word(&word)
}

pub(crate) fn is_known_en_token(token: &str) -> bool {
    if is_protected_ascii_token(token) {
        return true;
    }
    let Some(word) = normalized_token_core(token, Lang::En) else {
        return false;
    };
    en_dictionary().contains(&word)
}

pub(crate) fn all_tokens_known(text: &str, lang: Lang) -> bool {
    let mut found = false;
    for (segment, is_ws) in split_ws_segments(text) {
        if is_ws {
            continue;
        }
        let Some(word) = normalized_token_core(segment, lang) else {
            return false;
        };
        found = true;
        let known = match lang {
            Lang::Ru => is_known_ru_word(&word),
            Lang::En => en_dictionary().contains(&word),
        };
        if !known {
            return false;
        }
    }
    found
}

fn normalized_token_core(token: &str, lang: Lang) -> Option<String> {
    let word = token
        .trim_matches(|ch: char| !ch.is_alphabetic() && ch != '-')
        .to_lowercase();
    if word.is_empty() {
        return None;
    }

    let valid = match lang {
        Lang::Ru => word.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё' | '-')),
        Lang::En => word.chars().all(|ch| ch.is_ascii_alphabetic() || ch == '-'),
    };
    valid.then_some(word)
}

fn is_known_ru_word(word: &str) -> bool {
    is_ru_one_letter_function_word(word)
        || is_ru_single_letter_pronoun(word)
        || russian_tiny_dictionary().contains(word)
        || is_known_russian_word_or_form(word)
}

fn en_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, Lang::En).unwrap_or_default();
        words.extend(load_plain_words(EN_WORDS, Lang::En).unwrap_or_default());
        words
    })
}

fn load_hunspell_words(path: &str, lang: Lang) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .skip(1)
        .filter_map(|line| normalized_token_core(line.split('/').next().unwrap_or(""), lang))
        .collect())
}

fn load_plain_words(path: &str, lang: Lang) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .filter_map(|line| normalized_token_core(line, lang))
        .collect())
}

#[cfg(test)]
#[path = "token_language_tests.rs"]
mod tests;
