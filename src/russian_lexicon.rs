//! Russian lexical data facade.
//!
//! Dictionary caches, Hunspell loading and conservative form checks are split
//! into submodules. Correction rules should ask this facade whether a word is
//! known instead of embedding dictionary logic in the typing pipeline.

use crate::lexicon::{extend_common_ru_words, PROTECTED_WORDS_PATH, RU_HUNSPELL, RU_HUNSPELL_AFF};
use std::collections::HashSet;
use std::sync::OnceLock;

mod forms;
mod hunspell;

pub(crate) use forms::{
    is_known_cyrillic_hyphen_part, is_known_russian_adverb_o_form,
    is_known_russian_ka_oblique_form, looks_like_russian_adjective_lemma,
};
use hunspell::{
    load_hunspell_generated_forms_min_len, load_hunspell_words_min_len, load_word_list,
};

pub fn warm_up() {
    let _ = russian_dictionary().len();
    let _ = russian_short_dictionary().len();
    let _ = russian_tiny_dictionary().len();
    let _ = russian_generated_form_dictionary().len();
}

pub fn russian_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words_min_len(RU_HUNSPELL, 5).unwrap_or_default();
        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::PathBuf::from(home).join(PROTECTED_WORDS_PATH);
            if let Ok(custom) = load_word_list(&path) {
                words.extend(custom);
            }
        }
        #[cfg(test)]
        words.extend(crate::typing_assist_test_fixtures::russian_forms().map(str::to_string));
        extend_common_ru_words(&mut words);
        words
    })
}

pub fn russian_short_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let words = load_hunspell_words_min_len(RU_HUNSPELL, 3).unwrap_or_default();
        #[cfg(test)]
        {
            let mut words = words;
            words.extend(crate::typing_assist_test_fixtures::russian_forms().map(str::to_string));
            extend_common_ru_words(&mut words);
            words
        }
        #[cfg(not(test))]
        {
            let mut words = words;
            extend_common_ru_words(&mut words);
            words
        }
    })
}

pub fn russian_tiny_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let words = load_hunspell_words_min_len(RU_HUNSPELL, 2).unwrap_or_default();
        #[cfg(test)]
        {
            let mut words = words;
            words.extend(crate::typing_assist_test_fixtures::russian_forms().map(str::to_string));
            extend_common_ru_words(&mut words);
            words
        }
        #[cfg(not(test))]
        {
            let mut words = words;
            extend_common_ru_words(&mut words);
            words
        }
    })
}

pub fn russian_generated_form_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        load_hunspell_generated_forms_min_len(RU_HUNSPELL, RU_HUNSPELL_AFF, 4).unwrap_or_default()
    })
}

pub fn is_known_russian_word_or_form(word: &str) -> bool {
    russian_dictionary().contains(word)
        || russian_generated_form_dictionary().contains(word)
        || forms::is_known_russian_form(word)
}

#[cfg(test)]
#[path = "russian_lexicon_tests.rs"]
mod tests;
