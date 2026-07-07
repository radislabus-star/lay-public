//! Shared lexical data access.
//!
//! The correction engine must not embed word lists in production rules. Keep
//! lexical data in `data/lexicon/*` and expose it through small, hot `OnceLock`
//! sets so runtime checks stay cheap and platform-neutral.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

use crate::data_lines::data_lines;

pub const RU_HUNSPELL: &str = "/usr/share/hunspell/ru_RU.dic";
pub const RU_HUNSPELL_AFF: &str = "/usr/share/hunspell/ru_RU.aff";
pub const EN_HUNSPELL: &str = "/usr/share/hunspell/en_US.dic";
pub const EN_WORDS: &str = "/usr/share/dict/words";
pub const PROTECTED_WORDS_PATH: &str = ".config/lay/protected_words.txt";

const COMMON_RU_DATA: &str = include_str!("../data/lexicon/common_ru.txt");
const RU_TECHNICAL_LOANWORDS_DATA: &str =
    include_str!("../data/lexicon/ru_technical_loanwords.txt");
const RU_TECHNICAL_LOANWORD_STEMS_DATA: &str =
    include_str!("../data/lexicon/ru_technical_loanword_stems.txt");
const RU_TECHNICAL_LOANWORD_SUFFIXES_DATA: &str =
    include_str!("../data/lexicon/ru_technical_loanword_suffixes.txt");
const RU_LIVE_PROTECTED_WORDS_DATA: &str =
    include_str!("../data/lexicon/ru_live_protected_words.txt");
const COMMON_EN_TECHNICAL_DATA: &str = include_str!("../data/lexicon/common_en_technical.txt");
const COMMON_EN_GUARD_PREFIX_DATA: &str =
    include_str!("../data/lexicon/common_en_guard_prefixes.txt");
const RU_ONE_LETTER_FUNCTION_DATA: &str =
    include_str!("../data/lexicon/ru_one_letter_function.txt");
const RU_SINGLE_LETTER_PRONOUN_DATA: &str =
    include_str!("../data/lexicon/ru_single_letter_pronouns.txt");
const RU_SHORT_PRONOUN_DATA: &str = include_str!("../data/lexicon/ru_short_pronouns.txt");
const RU_SHORT_PREPOSITION_DATA: &str = include_str!("../data/lexicon/ru_short_prepositions.txt");
const RU_SHORT_FUNCTION_DATA: &str = include_str!("../data/lexicon/ru_short_function.txt");
const RU_HYPHEN_PARTICLE_DATA: &str = include_str!("../data/lexicon/ru_hyphen_particles.txt");
const RU_GREETING_WORDS_DATA: &str = include_str!("../data/lexicon/ru_greeting_words.txt");
const VISUAL_B_DEFAULT_DATA: &str = include_str!("../data/lexicon/visual_b_default.txt");
const VISUAL_B_AFTER_ASCII_DATA: &str = include_str!("../data/lexicon/visual_b_after_ascii.txt");
const IME_HOT_RU_DATA: &str = include_str!("../data/lexicon/l2_surface_hot_ru.txt");
const IME_HOT_PREFIX_BUCKET_LIMIT: usize = 96;
const IME_HOT_MIN_WORD_CHARS: usize = 4;

pub fn warm_up() {
    let _ = common_ru_words().len();
    let _ = ru_technical_loanwords().len();
    let _ = ru_live_protected_words().len();
    let _ = common_en_technical_words().len();
    let _ = common_en_technical_prefix_index().len();
    let _ = common_en_guard_prefixes().len();
    let _ = ru_one_letter_function_words().len();
    let _ = ru_single_letter_pronouns().len();
    let _ = ru_short_pronouns().len();
    let _ = ru_short_prepositions().len();
    let _ = ru_short_function_words().len();
    let _ = ru_hyphen_particles().len();
    let _ = ru_greeting_words().len();
    let _ = visual_b_default_replacement();
    let _ = visual_b_after_ascii_replacement();
    let _ = user_protected_words().len();
    let _ = user_protected_ascii_words().len();
}

pub fn warm_up_for_ime() {
    let _ = common_ru_words().len();
    let _ = ime_hot_ru_words().len();
    let _ = ime_hot_ru_prefix_index().len();
    let _ = common_en_technical_words().len();
    let _ = common_en_technical_prefix_index().len();
    let _ = common_en_guard_prefixes().len();
    let _ = ru_one_letter_function_words().len();
    let _ = ru_single_letter_pronouns().len();
    let _ = ru_short_pronouns().len();
    let _ = ru_short_prepositions().len();
    let _ = ru_short_function_words().len();
    let _ = ru_hyphen_particles().len();
    let _ = ru_greeting_words().len();
    let _ = visual_b_default_replacement();
    let _ = visual_b_after_ascii_replacement();
}

pub fn is_common_ru_word(word: &str) -> bool {
    common_ru_words().contains(word)
}

pub fn is_ime_hot_ru_word(word: &str) -> bool {
    ime_hot_ru_word_set().contains(word)
}

pub fn is_ru_technical_loanword(word: &str) -> bool {
    ru_technical_loanwords().contains(&word.trim().to_lowercase())
}

pub fn is_ru_live_protected_word(word: &str) -> bool {
    ru_live_protected_words().contains(&word.trim().to_lowercase())
}

pub fn common_ru_prefix_completion(prefix: &str, max_suffix_chars: usize) -> Option<String> {
    let prefix = prefix.trim().to_lowercase();
    if prefix.is_empty() {
        return None;
    }
    let word = common_ru_prefix_completion_word(&prefix, max_suffix_chars)?;
    word.get(prefix.len()..).map(str::to_string)
}

pub fn common_ru_prefix_completion_word(prefix: &str, max_suffix_chars: usize) -> Option<String> {
    let prefix = prefix.trim().to_lowercase();
    if prefix.is_empty() {
        return None;
    }
    common_ru_prefix_index()
        .get(&prefix)
        .into_iter()
        .flatten()
        .find(|word| word.chars().count() - prefix.chars().count() <= max_suffix_chars)
        .cloned()
}

pub fn common_ru_prefix_completion_words(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
) -> Vec<String> {
    let prefix = prefix.trim().to_lowercase();
    if prefix.is_empty() || limit == 0 {
        return Vec::new();
    }
    let prefix_len = prefix.chars().count();
    let mut words = common_ru_prefix_index()
        .get(&prefix)
        .into_iter()
        .flatten()
        .filter(|word| word.chars().count() - prefix.chars().count() <= max_suffix_chars)
        .cloned()
        .collect::<Vec<_>>();
    words.sort_by(|left, right| {
        left.chars()
            .count()
            .saturating_sub(prefix_len)
            .cmp(&right.chars().count().saturating_sub(prefix_len))
            .then_with(|| left.cmp(right))
    });
    words.truncate(limit);
    if words.len() >= limit {
        return words;
    }

    let mut seen = words.iter().cloned().collect::<HashSet<_>>();
    for word in hunspell_ru_prefix_completion_words(&prefix, max_suffix_chars, limit - words.len())
    {
        if seen.insert(word.clone()) {
            words.push(word);
            if words.len() >= limit {
                break;
            }
        }
    }
    words
}

pub fn ime_ru_prefix_completion_words(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
) -> Vec<String> {
    let prefix = prefix.trim().to_lowercase();
    if prefix.is_empty() || limit == 0 {
        return Vec::new();
    }
    let prefix_len = prefix.chars().count();
    ime_hot_ru_prefix_index()
        .get(&prefix)
        .into_iter()
        .flatten()
        .filter(|word| word.chars().count().saturating_sub(prefix_len) <= max_suffix_chars)
        .take(limit)
        .cloned()
        .collect()
}

pub fn common_en_technical_prefix_completion(
    prefix: &str,
    max_suffix_chars: usize,
) -> Option<String> {
    common_en_technical_prefix_completions(prefix, max_suffix_chars, 1)
        .into_iter()
        .next()
}

pub fn common_en_technical_prefix_completions(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
) -> Vec<String> {
    let prefix = prefix.trim().to_ascii_lowercase();
    if limit == 0
        || prefix.chars().count() < 2
        || !prefix.chars().all(|ch| ch.is_ascii_alphabetic())
    {
        return Vec::new();
    }
    common_en_technical_prefix_index()
        .get(&prefix)
        .into_iter()
        .flatten()
        .filter_map(|word| word.get(prefix.len()..))
        .filter(|suffix| suffix.chars().count() <= max_suffix_chars)
        .take(limit)
        .map(str::to_string)
        .collect()
}

pub fn is_common_en_technical_word(word: &str) -> bool {
    common_en_technical_words().contains(word)
}

pub fn is_common_en_guard_prefix(word: &str) -> bool {
    common_en_guard_prefixes().contains(word)
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

pub fn is_ru_greeting_word(word: &str) -> bool {
    ru_greeting_words().contains(&word.to_lowercase())
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

pub fn is_user_protected_word(word: &str) -> bool {
    let normalized = word.trim().to_lowercase();
    !normalized.is_empty() && user_protected_words().contains(&normalized)
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

pub fn extend_ru_technical_loanwords(words: &mut HashSet<String>) {
    words.extend(ru_technical_loanwords().iter().cloned());
}

pub fn common_ru_words_iter() -> impl Iterator<Item = &'static str> {
    common_ru_words_ordered().iter().map(String::as_str)
}

fn user_protected_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let Some(home) = std::env::var_os("HOME") else {
            return HashSet::new();
        };
        let path = std::path::PathBuf::from(home).join(PROTECTED_WORDS_PATH);
        load_plain_words(&path)
            .map(|words| {
                words
                    .into_iter()
                    .map(|word| word.trim().to_lowercase())
                    .filter(|word| !word.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    })
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

fn ru_technical_loanwords() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = parse_word_data(RU_TECHNICAL_LOANWORDS_DATA);
        for stem in data_lines(RU_TECHNICAL_LOANWORD_STEMS_DATA) {
            for suffix in data_lines(RU_TECHNICAL_LOANWORD_SUFFIXES_DATA) {
                words.insert(format!("{stem}{suffix}"));
            }
        }
        words
    })
}

fn ru_live_protected_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_LIVE_PROTECTED_WORDS_DATA))
}

fn common_ru_words_ordered() -> &'static Vec<String> {
    static WORDS: OnceLock<Vec<String>> = OnceLock::new();
    WORDS.get_or_init(|| data_lines(COMMON_RU_DATA).map(str::to_lowercase).collect())
}

fn common_ru_prefix_index() -> &'static HashMap<String, Vec<String>> {
    static INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    INDEX.get_or_init(|| build_prefix_index(common_ru_words_ordered()))
}

fn ime_hot_ru_words() -> &'static Vec<String> {
    static WORDS: OnceLock<Vec<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = common_ru_words_ordered().clone();
        words.extend(
            data_lines(IME_HOT_RU_DATA)
                .map(str::to_lowercase)
                .filter(|word| word.chars().count() >= IME_HOT_MIN_WORD_CHARS),
        );
        words.sort();
        words.dedup();
        words
    })
}

fn ime_hot_ru_word_set() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| ime_hot_ru_words().iter().cloned().collect())
}

fn ime_hot_ru_prefix_index() -> &'static HashMap<String, Vec<String>> {
    static INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    INDEX
        .get_or_init(|| build_bounded_prefix_index(ime_hot_ru_words(), IME_HOT_PREFIX_BUCKET_LIMIT))
}

fn hunspell_ru_words_ordered() -> &'static Vec<String> {
    static WORDS: OnceLock<Vec<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let Ok(text) = std::fs::read_to_string(RU_HUNSPELL) else {
            return Vec::new();
        };
        let mut words = parse_hunspell_ru_words(&text);
        words.sort();
        words.dedup();
        words
    })
}

fn hunspell_ru_prefix_completion_words(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let words = hunspell_ru_words_ordered();
    let start = words.partition_point(|word| word.as_str() < prefix);
    let prefix_len = prefix.chars().count();
    let mut result = Vec::with_capacity(limit.min(8));
    for word in words.iter().skip(start) {
        if !word.starts_with(prefix) {
            break;
        }
        if word.chars().count().saturating_sub(prefix_len) <= max_suffix_chars {
            result.push(word.clone());
            if result.len() >= limit {
                break;
            }
        }
    }
    result
}

fn common_en_technical_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(COMMON_EN_TECHNICAL_DATA))
}

fn common_en_technical_words_ordered() -> &'static Vec<String> {
    static WORDS: OnceLock<Vec<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        data_lines(COMMON_EN_TECHNICAL_DATA)
            .map(str::to_lowercase)
            .collect()
    })
}

fn common_en_technical_prefix_index() -> &'static HashMap<String, Vec<String>> {
    static INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    INDEX.get_or_init(|| build_prefix_index(common_en_technical_words_ordered()))
}

fn build_prefix_index(words: &[String]) -> HashMap<String, Vec<String>> {
    let mut index = HashMap::<String, Vec<String>>::new();
    for word in words {
        let char_count = word.chars().count();
        for prefix_len in 1..char_count {
            let prefix = word.chars().take(prefix_len).collect::<String>();
            index.entry(prefix).or_default().push(word.clone());
        }
    }
    index
}

fn build_bounded_prefix_index(
    words: &[String],
    bucket_limit: usize,
) -> HashMap<String, Vec<String>> {
    let mut ordered = words.to_vec();
    ordered.sort_by(|left, right| {
        is_common_ru_word(right)
            .cmp(&is_common_ru_word(left))
            .then_with(|| left.chars().count().cmp(&right.chars().count()))
            .then_with(|| left.cmp(right))
    });

    let mut index = HashMap::<String, Vec<String>>::new();
    for word in &ordered {
        let char_count = word.chars().count();
        for prefix_len in 1..char_count {
            let prefix = word.chars().take(prefix_len).collect::<String>();
            let bucket = index.entry(prefix).or_default();
            if bucket.len() < bucket_limit {
                bucket.push(word.clone());
            }
        }
    }
    index
}

fn common_en_guard_prefixes() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(COMMON_EN_GUARD_PREFIX_DATA))
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

fn ru_greeting_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| parse_word_data(RU_GREETING_WORDS_DATA))
}

fn parse_word_data(data: &str) -> HashSet<String> {
    data_lines(data).map(str::to_lowercase).collect()
}

fn parse_hunspell_ru_words(data: &str) -> Vec<String> {
    data.lines()
        .skip(1)
        .filter_map(|line| line.split('/').next())
        .map(str::trim)
        .filter(|word| {
            let len = word.chars().count();
            (3..=24).contains(&len)
                && word.chars().next().is_some_and(char::is_lowercase)
                && word.chars().all(is_russian_letter)
        })
        .map(str::to_lowercase)
        .collect()
}

fn is_russian_letter(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё' | 'А'..='Я' | 'Ё')
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
