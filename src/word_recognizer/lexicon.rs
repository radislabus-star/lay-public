use std::collections::HashSet;
use std::sync::OnceLock;

use crate::data_lines::data_lines;
use crate::lexicon::{
    extend_user_protected_ascii_words, is_common_en_technical_word, is_common_ru_word,
    is_ru_one_letter_function_word, is_ru_single_letter_pronoun, EN_HUNSPELL, EN_WORDS,
};
use crate::russian_lexicon::{is_known_russian_word_or_form, russian_tiny_dictionary};

use super::identity::WordScript;
use super::script::detect_script;

pub(super) fn warm_up() {
    crate::russian_lexicon::warm_up();
    let _ = english_words().len();
}

pub(super) fn known_russian_word(core: &str) -> bool {
    let lower = core.to_lowercase();
    if is_known_russian_core(&lower) {
        return true;
    }
    let parts: Vec<&str> = lower.split('-').filter(|part| !part.is_empty()).collect();
    parts.len() > 1 && parts.iter().all(|part| is_known_russian_core(part))
}

pub(super) fn known_english_word(core: &str) -> bool {
    if !core.is_ascii() {
        return false;
    }
    let lower = core.to_ascii_lowercase();
    if is_common_en_technical_word(&lower) || english_words().contains(&lower) {
        return true;
    }
    let parts: Vec<&str> = lower.split('-').filter(|part| !part.is_empty()).collect();
    parts.len() > 1 && parts.iter().all(|part| english_words().contains(*part))
}

fn is_known_russian_core(word: &str) -> bool {
    is_ru_one_letter_function_word(word)
        || is_ru_single_letter_pronoun(word)
        || is_common_ru_word(word)
        || russian_tiny_dictionary().contains(word)
        || is_known_russian_word_or_form(word)
}

fn english_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, 2, WordScript::Ascii);
        words.extend(load_plain_words(EN_WORDS, 2, WordScript::Ascii));
        extend_user_protected_ascii_words(&mut words, 1);
        words
    })
}

fn load_hunspell_words(path: &str, min_chars: usize, script: WordScript) -> HashSet<String> {
    let Ok(data) = std::fs::read_to_string(path) else {
        return HashSet::new();
    };
    collect_known_words(
        data_lines(&data).filter_map(|line| line.split('/').next()),
        min_chars,
        script,
    )
}

fn load_plain_words(path: &str, min_chars: usize, script: WordScript) -> HashSet<String> {
    let Ok(data) = std::fs::read_to_string(path) else {
        return HashSet::new();
    };
    collect_known_words(data_lines(&data), min_chars, script)
}

fn collect_known_words<'a>(
    words: impl Iterator<Item = &'a str>,
    min_chars: usize,
    script: WordScript,
) -> HashSet<String> {
    words
        .filter(|word| word.chars().count() >= min_chars)
        .filter(|word| detect_script(word) == script)
        .map(|word| word.to_lowercase())
        .collect()
}
