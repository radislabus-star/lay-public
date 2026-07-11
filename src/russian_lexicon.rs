//! Russian lexical data facade.
//!
//! Dictionary caches, Hunspell loading and conservative form checks are split
//! into submodules. Correction rules should ask this facade whether a word is
//! known instead of embedding dictionary logic in the typing pipeline.

use crate::lexicon::{
    extend_common_ru_words, extend_ru_technical_loanwords, PROTECTED_WORDS_PATH, RU_HUNSPELL,
    RU_HUNSPELL_AFF,
};
use std::sync::OnceLock;

mod forms;
mod hunspell;
mod word_set;

pub(crate) use forms::{
    is_center_backed_russian_form, is_known_cyrillic_hyphen_part, is_known_russian_adverb_o_form,
    is_known_russian_ka_oblique_form, is_reference_backed_russian_form,
    looks_like_russian_adjective_lemma,
};
use hunspell::{
    load_hunspell_generated_forms_min_len, load_hunspell_words_min_len, load_word_list,
};
pub(crate) use word_set::WordSet;

static RUSSIAN_DICTIONARY: OnceLock<WordSet> = OnceLock::new();
static RUSSIAN_SHORT_DICTIONARY: OnceLock<WordSet> = OnceLock::new();
static RUSSIAN_TINY_DICTIONARY: OnceLock<WordSet> = OnceLock::new();
static RUSSIAN_GENERATED_FORMS: OnceLock<WordSet> = OnceLock::new();
static EMPTY_WORD_SET: OnceLock<WordSet> = OnceLock::new();
static EMPTY_GENERATED_FORMS: OnceLock<WordSet> = OnceLock::new();

pub fn warm_up() {
    let _ = russian_dictionary().len();
    let _ = russian_short_dictionary().len();
    let _ = russian_tiny_dictionary().len();
}

pub fn russian_dictionary() -> &'static WordSet {
    russian_dictionary_for_authority(crate::hot_field::process_policy().authority())
}

fn russian_dictionary_for_authority(authority: crate::hot_field::HotAuthority) -> &'static WordSet {
    if matches!(authority, crate::hot_field::HotAuthority::FieldSnapshotOnly) {
        empty_word_set()
    } else {
        full_russian_dictionary()
    }
}

fn full_russian_dictionary() -> &'static WordSet {
    RUSSIAN_DICTIONARY.get_or_init(|| {
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
        extend_ru_technical_loanwords(&mut words);
        WordSet::from_words(words)
    })
}

pub fn russian_short_dictionary() -> &'static WordSet {
    russian_short_dictionary_for_authority(crate::hot_field::process_policy().authority())
}

fn russian_short_dictionary_for_authority(
    authority: crate::hot_field::HotAuthority,
) -> &'static WordSet {
    if matches!(authority, crate::hot_field::HotAuthority::FieldSnapshotOnly) {
        empty_word_set()
    } else {
        full_russian_short_dictionary()
    }
}

fn full_russian_short_dictionary() -> &'static WordSet {
    RUSSIAN_SHORT_DICTIONARY.get_or_init(|| {
        let words = load_hunspell_words_min_len(RU_HUNSPELL, 3).unwrap_or_default();
        #[cfg(test)]
        {
            let mut words = words;
            words.extend(crate::typing_assist_test_fixtures::russian_forms().map(str::to_string));
            extend_common_ru_words(&mut words);
            extend_ru_technical_loanwords(&mut words);
            WordSet::from_words(words)
        }
        #[cfg(not(test))]
        {
            let mut words = words;
            extend_common_ru_words(&mut words);
            extend_ru_technical_loanwords(&mut words);
            WordSet::from_words(words)
        }
    })
}

pub fn russian_tiny_dictionary() -> &'static WordSet {
    russian_tiny_dictionary_for_authority(crate::hot_field::process_policy().authority())
}

fn russian_tiny_dictionary_for_authority(
    authority: crate::hot_field::HotAuthority,
) -> &'static WordSet {
    if matches!(authority, crate::hot_field::HotAuthority::FieldSnapshotOnly) {
        empty_word_set()
    } else {
        full_russian_tiny_dictionary()
    }
}

fn full_russian_tiny_dictionary() -> &'static WordSet {
    RUSSIAN_TINY_DICTIONARY.get_or_init(|| {
        let words = load_hunspell_words_min_len(RU_HUNSPELL, 2).unwrap_or_default();
        #[cfg(test)]
        {
            let mut words = words;
            words.extend(crate::typing_assist_test_fixtures::russian_forms().map(str::to_string));
            extend_common_ru_words(&mut words);
            extend_ru_technical_loanwords(&mut words);
            WordSet::from_words(words)
        }
        #[cfg(not(test))]
        {
            let mut words = words;
            extend_common_ru_words(&mut words);
            extend_ru_technical_loanwords(&mut words);
            WordSet::from_words(words)
        }
    })
}

pub fn russian_generated_form_dictionary() -> &'static WordSet {
    russian_generated_form_dictionary_for_authority(crate::hot_field::process_policy().authority())
}

fn russian_generated_form_dictionary_for_authority(
    authority: crate::hot_field::HotAuthority,
) -> &'static WordSet {
    if matches!(authority, crate::hot_field::HotAuthority::FieldSnapshotOnly)
        || !full_generated_forms_enabled()
    {
        EMPTY_GENERATED_FORMS.get_or_init(|| WordSet::from_words(Default::default()))
    } else {
        full_russian_generated_form_dictionary()
    }
}

fn full_russian_generated_form_dictionary() -> &'static WordSet {
    RUSSIAN_GENERATED_FORMS.get_or_init(|| {
        WordSet::from_words(
            load_hunspell_generated_forms_min_len(RU_HUNSPELL, RU_HUNSPELL_AFF, 4)
                .unwrap_or_default(),
        )
    })
}

pub fn russian_generated_form_dictionary_is_warm() -> bool {
    RUSSIAN_GENERATED_FORMS.get().is_some()
}

pub fn russian_dictionary_is_warm() -> bool {
    RUSSIAN_DICTIONARY.get().is_some()
}

fn full_generated_forms_enabled() -> bool {
    cfg!(test) || std::env::var_os("LAY_ENABLE_FULL_GENERATED_FORMS").is_some()
}

pub fn is_known_russian_word_or_form(word: &str) -> bool {
    if !crate::hot_field::process_allows_full_reference_authority() {
        return crate::hot_field::HotFieldSnapshot::current()
            .word_readout(word)
            .is_known()
            || forms::is_known_russian_form(word)
            || crate::lexicon::is_ru_technical_loanword(word);
    }

    full_russian_dictionary().contains(word)
        || (full_generated_forms_enabled()
            && full_russian_generated_form_dictionary().contains(word))
        || forms::is_known_russian_form(word)
        || crate::lexicon::is_ru_technical_loanword(word)
}

fn empty_word_set() -> &'static WordSet {
    EMPTY_WORD_SET.get_or_init(|| WordSet::from_words(Default::default()))
}

#[cfg(test)]
#[path = "russian_lexicon_tests.rs"]
mod tests;
