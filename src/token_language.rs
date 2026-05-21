//! Token-level language recognition for layout arbiters.
//!
//! This is a small lexical layer: normalize a token, decide whether it is known
//! Russian/English, and evaluate whole-token sequences. It does not correct or
//! emit text.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::lexicon::{extend_common_ru_words, EN_HUNSPELL, EN_WORDS, RU_HUNSPELL};
use crate::word_reader::split_ws_segments;
use crate::word_recognizer::is_protected_ascii_token;

#[derive(Clone, Copy)]
pub(crate) enum Lang {
    Ru,
    En,
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
    if ru_dictionary().contains(word) {
        return true;
    }
    let len = word.chars().count();
    if len < 4 {
        return false;
    }

    const SUFFIXES: &[&str] = &[
        "ыми", "ими", "ами", "ями", "ого", "его", "ому", "ему", "ах", "ях", "ам", "ям", "ом", "ем",
        "ой", "ей", "ый", "ий", "ая", "яя", "ое", "ее", "ые", "ие", "а", "я", "у", "ю", "е", "ы",
        "и",
    ];
    SUFFIXES.iter().any(|suffix| {
        let Some(stem) = word.strip_suffix(suffix) else {
            return false;
        };
        stem.chars().count() >= 3 && ru_dictionary().contains(stem)
    }) || is_known_ru_verb_form(word)
}

fn is_known_ru_verb_form(word: &str) -> bool {
    for (ending, lemmas) in [("айте", &["ать"][..]), ("ай", &["ать"][..])] {
        let Some(stem) = word.strip_suffix(ending) else {
            continue;
        };
        if stem.chars().count() >= 3
            && lemmas
                .iter()
                .any(|lemma_suffix| ru_dictionary().contains(&format!("{stem}{lemma_suffix}")))
        {
            return true;
        }
    }
    false
}

fn ru_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(RU_HUNSPELL, Lang::Ru).unwrap_or_default();
        extend_common_ru_words(&mut words);
        words
    })
}

fn en_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, Lang::En).unwrap_or_default();
        if words.is_empty() {
            words.extend(load_plain_words(EN_WORDS, Lang::En).unwrap_or_default());
        }
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
