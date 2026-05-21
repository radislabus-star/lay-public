//! Word recognition layer for correction safety.
//!
//! This module does not correct text. It classifies a token so higher layers can
//! decide whether an automatic correction is safe enough to apply.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    is_common_en_technical_word, is_common_ru_word, EN_HUNSPELL, EN_WORDS, RU_HUNSPELL,
};
use crate::word_reader::split_word_punctuation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordScript {
    Empty,
    Cyrillic,
    Ascii,
    Mixed,
    Numeric,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    Empty,
    PlainWord,
    TechnicalToken,
    CliOption,
    Number,
    MixedScript,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordLanguage {
    Russian,
    English,
    Ambiguous,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordIdentity<'a> {
    pub token: &'a str,
    pub core: &'a str,
    pub kind: WordKind,
    pub script: WordScript,
    pub language: WordLanguage,
    pub known_ru: bool,
    pub known_en: bool,
    pub protected: bool,
    pub technical: bool,
}

pub fn recognize_token(token: &str) -> WordIdentity<'_> {
    let (_, core, _) = split_word_punctuation(token);
    if core.is_empty() {
        return WordIdentity {
            token,
            core,
            kind: WordKind::Empty,
            script: WordScript::Empty,
            language: WordLanguage::Unknown,
            known_ru: false,
            known_en: false,
            protected: false,
            technical: false,
        };
    }

    let script = detect_script(core);
    let protected = is_protected_ascii_token(core);
    let cli_option = is_cli_option_token(token);
    let technical = cli_option || is_technical_token(core) || protected;
    let known_ru = known_russian_word(core);
    let known_en = known_english_word(core);
    let language = match (known_ru, known_en, script) {
        (true, true, _) => WordLanguage::Ambiguous,
        (true, false, _) => WordLanguage::Russian,
        (false, true, _) => WordLanguage::English,
        (_, _, WordScript::Cyrillic) => WordLanguage::Russian,
        (_, _, WordScript::Ascii) => WordLanguage::English,
        _ => WordLanguage::Unknown,
    };
    let kind = if cli_option {
        WordKind::CliOption
    } else if technical {
        WordKind::TechnicalToken
    } else if script == WordScript::Mixed {
        WordKind::MixedScript
    } else if script == WordScript::Numeric {
        WordKind::Number
    } else if matches!(script, WordScript::Cyrillic | WordScript::Ascii) {
        WordKind::PlainWord
    } else {
        WordKind::Other
    };

    WordIdentity {
        token,
        core,
        kind,
        script,
        language,
        known_ru,
        known_en,
        protected,
        technical,
    }
}

pub fn is_plain_layout_autocorrect_risky(original: &str, replacement: &str) -> bool {
    let original = recognize_token(original);
    let replacement = recognize_token(replacement);

    if original.kind == WordKind::Empty || replacement.kind == WordKind::Empty {
        return true;
    }
    if original.kind == WordKind::CliOption || replacement.kind == WordKind::CliOption {
        return true;
    }
    if original.technical || replacement.technical {
        return false;
    }
    if original.kind == WordKind::MixedScript || replacement.kind == WordKind::MixedScript {
        return false;
    }
    if original.script == WordScript::Cyrillic
        && replacement.script == WordScript::Ascii
        && !original.known_ru
        && replacement.known_en
    {
        return false;
    }

    matches!(
        (original.kind, replacement.kind),
        (WordKind::PlainWord, WordKind::PlainWord)
    )
}

pub fn is_probably_completed_natural_word(token: &str) -> bool {
    let identity = recognize_token(token);
    matches!(
        identity.kind,
        WordKind::PlainWord | WordKind::TechnicalToken
    ) && (identity.known_ru || identity.known_en || identity.technical)
}

fn detect_script(core: &str) -> WordScript {
    let mut has_cyrillic = false;
    let mut has_ascii = false;
    let mut has_digit = false;
    let mut has_other = false;

    for ch in core.chars() {
        if is_cyrillic_letter(ch) {
            has_cyrillic = true;
        } else if ch.is_ascii_alphabetic() {
            has_ascii = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if matches!(
            ch,
            '-' | '_'
                | '.'
                | '/'
                | '+'
                | ','
                | ';'
                | '\''
                | '['
                | ']'
                | '`'
                | '?'
                | '!'
                | ':'
                | '$'
                | '%'
                | '^'
                | '&'
                | '#'
                | '@'
        ) {
        } else {
            has_other = true;
        }
    }

    match (has_cyrillic, has_ascii, has_digit, has_other) {
        (false, false, false, _) => WordScript::Other,
        (true, false, false, false) => WordScript::Cyrillic,
        (false, true, false, false) => WordScript::Ascii,
        (false, false, true, false) => WordScript::Numeric,
        (true, true, _, false) => WordScript::Mixed,
        (true, false, true, false) => WordScript::Mixed,
        (false, true, true, false) => WordScript::Mixed,
        _ => WordScript::Other,
    }
}

fn is_technical_token(core: &str) -> bool {
    let lower = core.to_ascii_lowercase();
    if is_common_en_technical_word(&lower) {
        return true;
    }
    if core.contains("://") || core.contains('@') {
        return true;
    }
    if core.chars().any(|ch| ch.is_ascii_digit()) {
        return true;
    }
    if is_ascii_technical_or_brand_token(core) {
        return true;
    }
    false
}

pub fn is_cli_option_token(token: &str) -> bool {
    let rest = if let Some(rest) = token.strip_prefix("--") {
        rest
    } else if let Some(rest) = token.strip_prefix('-') {
        rest
    } else {
        return false;
    };

    !rest.is_empty()
        && rest.chars().any(|ch| ch.is_ascii_alphabetic())
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '=' | ':' | '.' | '/'))
}

pub fn is_protected_ascii_token(core: &str) -> bool {
    if !core.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    core.is_ascii()
        && (has_domain_like_dot(core)
            || core.contains('@')
            || core.contains("://")
            || core.contains('/')
            || core.contains('\\')
            || is_upper_ascii_acronym(core)
            || is_mixed_case_ascii_brand(core))
}

pub fn is_ascii_technical_token(core: &str) -> bool {
    core.is_ascii()
        && core.chars().any(|ch| ch.is_ascii_alphabetic())
        && core.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, '-' | '_' | '.' | '@' | '/' | '\\' | ':' | '+' | '#')
        })
        && core
            .chars()
            .any(|ch| matches!(ch, '-' | '_' | '.' | '@' | '/' | '\\' | ':' | '+' | '#'))
}

pub fn is_ascii_technical_or_brand_token(core: &str) -> bool {
    core.is_ascii()
        && core.chars().any(|ch| ch.is_ascii_alphabetic())
        && (has_domain_like_dot(core)
            || has_ascii_hyphen_or_underscore_segments(core)
            || core
                .chars()
                .any(|ch| matches!(ch, '@' | '/' | '\\' | ':' | '+' | '#'))
            || is_upper_ascii_acronym(core)
            || is_mixed_case_ascii_brand(core))
}

pub fn is_upper_ascii_acronym(core: &str) -> bool {
    let letters: Vec<char> = core.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    (2..=4).contains(&letters.len()) && letters.iter().all(|ch| ch.is_ascii_uppercase())
}

pub fn is_mixed_case_ascii_brand(core: &str) -> bool {
    let letters: Vec<char> = core.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    letters.len() >= 4
        && letters.iter().any(|ch| ch.is_ascii_lowercase())
        && letters.iter().skip(1).any(|ch| ch.is_ascii_uppercase())
}

pub fn is_mixed_cyrillic_ascii_alpha_token(core: &str) -> bool {
    let mut has_cyrillic = false;
    let mut has_ascii = false;
    for ch in core.chars() {
        if is_cyrillic_letter(ch) {
            has_cyrillic = true;
        } else if ch.is_ascii_alphabetic() {
            has_ascii = true;
        } else if !matches!(ch, '-' | '\'') {
            return false;
        }
    }
    has_cyrillic && has_ascii
}

fn has_domain_like_dot(core: &str) -> bool {
    core.split('.').count() >= 2
        && core.rsplit_once('.').is_some_and(|(name, tld)| {
            name.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2
                && (2..=4).contains(&tld.chars().count())
                && tld.chars().all(|ch| ch.is_ascii_alphabetic())
        })
}

fn has_ascii_hyphen_or_underscore_segments(core: &str) -> bool {
    core.split(['-', '_']).count() >= 2
        && core
            .split(['-', '_'])
            .all(|part| part.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2)
}

fn known_russian_word(core: &str) -> bool {
    let lower = core.to_lowercase();
    if is_common_ru_word(&lower) || russian_words().contains(&lower) {
        return true;
    }
    let parts: Vec<&str> = lower.split('-').filter(|part| !part.is_empty()).collect();
    parts.len() > 1 && parts.iter().all(|part| russian_words().contains(*part))
}

fn known_english_word(core: &str) -> bool {
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

fn russian_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| load_hunspell_words(RU_HUNSPELL, 2, WordScript::Cyrillic))
}

fn english_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, 2, WordScript::Ascii);
        if let Ok(extra) = std::fs::read_to_string(EN_WORDS) {
            words.extend(
                extra
                    .lines()
                    .map(str::trim)
                    .filter(|word| word.chars().count() >= 2)
                    .filter(|word| detect_script(word) == WordScript::Ascii)
                    .map(|word| word.to_ascii_lowercase()),
            );
        }
        words
    })
}

fn load_hunspell_words(path: &str, min_chars: usize, script: WordScript) -> HashSet<String> {
    let Ok(data) = std::fs::read_to_string(path) else {
        return HashSet::new();
    };
    data.lines()
        .filter_map(|line| line.split('/').next())
        .map(str::trim)
        .filter(|word| word.chars().count() >= min_chars)
        .filter(|word| detect_script(word) == script)
        .map(|word| word.to_lowercase())
        .collect()
}

#[cfg(test)]
#[path = "word_recognizer_tests.rs"]
mod tests;
