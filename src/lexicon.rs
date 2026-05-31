//! Shared lexical data access.
//!
//! The correction engine must not embed word lists in production rules. Keep
//! lexical data in `data/lexicon/*` and expose it through small, hot `OnceLock`
//! sets so runtime checks stay cheap and platform-neutral.

use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;

use crate::data_lines::data_lines;

pub const RU_HUNSPELL: &str = "/usr/share/hunspell/ru_RU.dic";
pub const RU_HUNSPELL_AFF: &str = "/usr/share/hunspell/ru_RU.aff";
pub const EN_HUNSPELL: &str = "/usr/share/hunspell/en_US.dic";
pub const EN_WORDS: &str = "/usr/share/dict/words";
pub const PROTECTED_WORDS_PATH: &str = ".config/lay/protected_words.txt";

const COMMON_RU_DATA: &str = include_str!("../data/lexicon/common_ru.txt");
const COMMON_EN_TECHNICAL_DATA: &str = include_str!("../data/lexicon/common_en_technical.txt");
const RU_ONE_LETTER_FUNCTION_DATA: &str =
    include_str!("../data/lexicon/ru_one_letter_function.txt");
const RU_SINGLE_LETTER_PRONOUN_DATA: &str =
    include_str!("../data/lexicon/ru_single_letter_pronouns.txt");
const RU_SHORT_PRONOUN_DATA: &str = include_str!("../data/lexicon/ru_short_pronouns.txt");
const RU_SHORT_PREPOSITION_DATA: &str = include_str!("../data/lexicon/ru_short_prepositions.txt");
const RU_SHORT_FUNCTION_DATA: &str = include_str!("../data/lexicon/ru_short_function.txt");
const RU_HYPHEN_PARTICLE_DATA: &str = include_str!("../data/lexicon/ru_hyphen_particles.txt");
const VISUAL_B_DEFAULT_DATA: &str = include_str!("../data/lexicon/visual_b_default.txt");
const VISUAL_B_AFTER_ASCII_DATA: &str = include_str!("../data/lexicon/visual_b_after_ascii.txt");

pub fn warm_up() {
    let _ = common_ru_words().len();
    let _ = common_en_technical_words().len();
    let _ = ru_one_letter_function_words().len();
    let _ = ru_single_letter_pronouns().len();
    let _ = ru_short_pronouns().len();
    let _ = ru_short_prepositions().len();
    let _ = ru_short_function_words().len();
    let _ = ru_hyphen_particles().len();
    let _ = visual_b_default_replacement();
    let _ = visual_b_after_ascii_replacement();
    let _ = user_protected_ascii_words().len();
}

pub fn is_common_ru_word(word: &str) -> bool {
    common_ru_words().contains(word)
}

pub fn is_common_en_technical_word(word: &str) -> bool {
    common_en_technical_words().contains(word)
}

pub fn is_ru_one_letter_function_word(word: &str) -> bool {
    ru_one_letter_function_words().contains(word)
}

pub fn is_ru_single_letter_pronoun(word: &str) -> bool {
    ru_single_letter_pronouns().contains(word)
}

pub fn is_ru_short_pronoun(word: &str) -> bool {
    ru_short_pronouns().contains(word)
}

pub fn is_ru_short_preposition(word: &str) -> bool {
    ru_short_prepositions().contains(word)
}

pub fn is_ru_short_function_word(word: &str) -> bool {
    ru_short_function_words().contains(word)
}

pub fn is_ru_hyphen_particle(word: &str) -> bool {
    ru_hyphen_particles().contains(word)
}

pub fn visual_b_default_replacement() -> &'static str {
    first_data_word(VISUAL_B_DEFAULT_DATA)
}

pub fn visual_b_after_ascii_replacement() -> &'static str {
    first_data_word(VISUAL_B_AFTER_ASCII_DATA)
}

pub fn is_user_protected_ascii_word(word: &str) -> bool {
    if !word.is_ascii() {
        return false;
    }
    user_protected_ascii_words().contains(&word.to_ascii_lowercase())
}

pub fn extend_user_protected_ascii_words(words: &mut HashSet<String>, min_chars: usize) {
    words.extend(
        user_protected_ascii_words()
            .iter()
            .filter(|word| word.chars().count() >= min_chars)
            .cloned(),
    );
}

pub fn extend_common_ru_words(words: &mut HashSet<String>) {
    words.extend(common_ru_words().iter().cloned());
}

fn user_protected_ascii_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let Some(home) = std::env::var_os("HOME") else {
            return HashSet::new();
        };
        let path = std::path::PathBuf::from(home).join(PROTECTED_WORDS_PATH);
        load_plain_words(&path)
            .map(|words| ascii_words_from_iter(words, 1))
            .unwrap_or_default()
    })
}

fn common_ru_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(COMMON_RU_DATA))
}

fn common_en_technical_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(COMMON_EN_TECHNICAL_DATA))
}

fn ru_one_letter_function_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_ONE_LETTER_FUNCTION_DATA))
}

fn ru_single_letter_pronouns() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_SINGLE_LETTER_PRONOUN_DATA))
}

fn ru_short_pronouns() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_SHORT_PRONOUN_DATA))
}

fn ru_short_prepositions() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_SHORT_PREPOSITION_DATA))
}

fn ru_short_function_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_SHORT_FUNCTION_DATA))
}

fn ru_hyphen_particles() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_HYPHEN_PARTICLE_DATA))
}

fn parse_word_data(data: &str) -> HashSet<String> {
    data_lines(data).map(str::to_lowercase).collect()
}

#[cfg(test)]
pub(crate) fn parse_ascii_word_data(data: &str, min_chars: usize) -> HashSet<String> {
    ascii_words_from_iter(data_lines(data).map(str::to_string), min_chars)
}

fn ascii_words_from_iter<I>(words: I, min_chars: usize) -> HashSet<String>
where
    I: IntoIterator<Item = String>,
{
    words
        .into_iter()
        .map(|word| word.trim().to_ascii_lowercase())
        .filter(|word| word.chars().count() >= min_chars)
        .filter(|word| {
            word.is_ascii()
                && word
                    .chars()
                    .all(|ch| ch.is_ascii_alphabetic() || ch == '-' || ch == '_')
        })
        .collect()
}

fn load_plain_words(path: &Path) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(parse_word_data(&text))
}

fn first_data_word(data: &'static str) -> &'static str {
    data_lines(data).next().unwrap_or("")
}

#[cfg(test)]
#[path = "lexicon_tests.rs"]
mod tests;
