use crate::word_reader::split_word_punctuation;

use super::lexicon::{known_english_word, known_russian_word};
use super::script::detect_script;
use super::technical::{is_cli_option_token, is_protected_ascii_token, is_technical_token};

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
        return empty_identity(token, core);
    }

    let script = detect_script(core);
    let protected = is_protected_ascii_token(core);
    let cli_option = is_cli_option_token(token);
    let technical = cli_option || is_technical_token(core) || protected;
    let known_ru = known_russian_word(core);
    let known_en = known_english_word(core);

    WordIdentity {
        token,
        core,
        kind: word_kind(script, cli_option, technical),
        script,
        language: word_language(known_ru, known_en, script),
        known_ru,
        known_en,
        protected,
        technical,
    }
}

fn empty_identity<'a>(token: &'a str, core: &'a str) -> WordIdentity<'a> {
    WordIdentity {
        token,
        core,
        kind: WordKind::Empty,
        script: WordScript::Empty,
        language: WordLanguage::Unknown,
        known_ru: false,
        known_en: false,
        protected: false,
        technical: false,
    }
}

fn word_language(known_ru: bool, known_en: bool, script: WordScript) -> WordLanguage {
    match (known_ru, known_en, script) {
        (true, true, _) => WordLanguage::Ambiguous,
        (true, false, _) => WordLanguage::Russian,
        (false, true, _) => WordLanguage::English,
        (_, _, WordScript::Cyrillic) => WordLanguage::Russian,
        (_, _, WordScript::Ascii) => WordLanguage::English,
        _ => WordLanguage::Unknown,
    }
}

fn word_kind(script: WordScript, cli_option: bool, technical: bool) -> WordKind {
    if cli_option {
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
    }
}
