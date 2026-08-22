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

static ENGLISH_WORDS: OnceLock<Box<[Box<str>]>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExactWordGuardReceipt {
    pub(crate) english_fingerprint: u64,
    pub(crate) protection_fingerprint: u64,
    pub(crate) english_entries: usize,
    pub(crate) protection_entries: usize,
    pub(crate) resident_bytes: usize,
}

pub(super) fn warm_up() {
    crate::russian_lexicon::warm_up();
    let _ = english_words().len();
}

pub(super) fn warm_up_exact_layout_guard() -> ExactWordGuardReceipt {
    let (protection_fingerprint, protection_entries, protection_bytes) =
        crate::lexicon::warm_up_exact_ascii_protection();
    let words = english_words();
    ExactWordGuardReceipt {
        english_fingerprint: fingerprint_words(words),
        protection_fingerprint,
        english_entries: words.len(),
        protection_entries,
        resident_bytes: words
            .iter()
            .map(|word| word.len())
            .sum::<usize>()
            .saturating_add(words.len().saturating_mul(std::mem::size_of::<Box<str>>()))
            .saturating_add(protection_bytes),
    }
}

pub(super) fn known_english_word_if_warm(core: &str) -> Option<bool> {
    if !core.is_ascii() {
        return Some(false);
    }
    let words = ENGLISH_WORDS.get()?.as_ref();
    let lower = core.to_ascii_lowercase();
    if crate::lexicon::is_common_en_technical_word_if_warm(&lower)?
        || exact_word_bank_contains(words, &lower)
    {
        return Some(true);
    }
    let parts = lower
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    Some(
        parts.len() > 1
            && parts
                .iter()
                .all(|part| exact_word_bank_contains(words, part)),
    )
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
    if is_common_en_technical_word(&lower) || exact_word_bank_contains(english_words(), &lower) {
        return true;
    }
    let parts: Vec<&str> = lower.split('-').filter(|part| !part.is_empty()).collect();
    parts.len() > 1
        && parts
            .iter()
            .all(|part| exact_word_bank_contains(english_words(), part))
}

fn is_known_russian_core(word: &str) -> bool {
    is_ru_one_letter_function_word(word)
        || is_ru_single_letter_pronoun(word)
        || is_common_ru_word(word)
        || russian_tiny_dictionary().contains(word)
        || is_known_russian_word_or_form(word)
}

fn english_words() -> &'static [Box<str>] {
    ENGLISH_WORDS
        .get_or_init(|| {
            let mut words = load_hunspell_words(EN_HUNSPELL, 2, WordScript::Ascii);
            words.extend(load_plain_words(EN_WORDS, 2, WordScript::Ascii));
            let mut protected = HashSet::new();
            extend_user_protected_ascii_words(&mut protected, 1);
            words.extend(protected.into_iter().map(String::into_boxed_str));
            words.sort_unstable();
            words.dedup();
            words.into_boxed_slice()
        })
        .as_ref()
}

fn exact_word_bank_contains(words: &[Box<str>], needle: &str) -> bool {
    words
        .binary_search_by(|word| word.as_ref().cmp(needle))
        .is_ok()
}

fn fingerprint_words(words: &[Box<str>]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for word in words {
        for byte in word.bytes().chain(std::iter::once(0)) {
            digest ^= u64::from(byte);
            digest = digest.wrapping_mul(0x100_0000_01b3);
        }
    }
    digest
}

fn load_hunspell_words(path: &str, min_chars: usize, script: WordScript) -> Vec<Box<str>> {
    let Ok(data) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    collect_known_words(
        data_lines(&data).filter_map(|line| line.split('/').next()),
        min_chars,
        script,
    )
}

fn load_plain_words(path: &str, min_chars: usize, script: WordScript) -> Vec<Box<str>> {
    let Ok(data) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    collect_known_words(data_lines(&data), min_chars, script)
}

fn collect_known_words<'a>(
    words: impl Iterator<Item = &'a str>,
    min_chars: usize,
    script: WordScript,
) -> Vec<Box<str>> {
    words
        .filter(|word| word.chars().count() >= min_chars)
        .filter(|word| detect_script(word) == script)
        .map(|word| word.to_lowercase().into_boxed_str())
        .collect()
}
