//! Russian lexical data and lightweight form recognition.
//!
//! This module owns Hunspell loading, generated forms and conservative
//! word-form checks. Correction rules should ask this module whether a word is
//! known instead of embedding dictionary logic in the typing pipeline.

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{extend_common_ru_words, PROTECTED_WORDS_PATH, RU_HUNSPELL, RU_HUNSPELL_AFF};
use crate::russian_chars::is_russian_vowel;
use crate::russian_prefixes::DERIVATIONAL_PREFIXES;
use crate::word_reader::is_cyrillic_word;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

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
            words.insert("пара".to_string());
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
            words.insert("не".to_string());
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
        || is_known_russian_suffix_form(word)
        || is_known_russian_zero_ending_noun_form(word)
        || is_known_russian_ka_declension_form(word)
        || is_known_russian_prefixed_form(word)
        || is_known_russian_verb_form(word)
}

pub(crate) fn is_known_russian_adverb_o_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    let Some(stem) = word.strip_suffix('о') else {
        return false;
    };
    if stem.chars().count() < 3 {
        return false;
    }

    ["ый", "ий", "ой"]
        .iter()
        .any(|suffix| russian_dictionary().contains(&format!("{stem}{suffix}")))
}

pub(crate) fn is_known_russian_ka_oblique_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    for suffix in ["ками", "ках", "кой", "ки", "ке", "ку"] {
        if let Some(stem) = word.strip_suffix(suffix) {
            return stem.chars().count() >= 3
                && russian_dictionary().contains(&format!("{stem}ка"));
        }
    }
    false
}

pub(crate) fn is_known_cyrillic_hyphen_part(part: &str, dict: &HashSet<String>) -> bool {
    let lower = part.to_lowercase();
    dict.contains(&lower)
        || russian_generated_form_dictionary().contains(&lower)
        || is_known_short_accusative_a_form(&lower, dict)
}

pub(crate) fn looks_like_russian_adjective_lemma(word: &str) -> bool {
    word.ends_with("ый") || word.ends_with("ий") || word.ends_with("ой")
}

fn is_known_russian_suffix_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }

    const SUFFIXES: &[&str] = &[
        "ыми", "ими", "ами", "ями", "ого", "его", "ому", "ему", "ов", "ев", "ей", "ах", "ях", "ам",
        "ям", "ом", "ем", "ой", "ый", "ий", "ая", "яя", "ое", "ее", "ые", "ие", "а", "я", "у", "ю",
        "е", "ы", "и",
    ];

    SUFFIXES.iter().any(|suffix| {
        let Some(stem) = word.strip_suffix(suffix) else {
            return false;
        };
        if stem.chars().count() < 3 {
            return false;
        }
        if matches!(*suffix, "ы" | "и") && looks_like_russian_adjective_lemma(stem) {
            return false;
        }
        if russian_dictionary().contains(stem) {
            return true;
        }
        matches!(*suffix, "ами" | "ями")
            && (russian_short_dictionary().contains(stem)
                || russian_dictionary().contains(&format!("{stem}о")))
    })
}

fn is_known_russian_zero_ending_noun_form(word: &str) -> bool {
    if word.chars().count() < 5 || !word.chars().last().is_some_and(is_russian_consonant) {
        return false;
    }

    ["а", "я"].iter().any(|suffix| {
        let lemma = format!("{word}{suffix}");
        russian_dictionary().contains(&lemma) || russian_short_dictionary().contains(&lemma)
    })
}

fn is_known_russian_ka_declension_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    let Some(stem) = word.strip_suffix("ок") else {
        return false;
    };
    stem.chars().count() >= 3 && russian_dictionary().contains(&format!("{stem}ка"))
}

fn is_known_russian_prefixed_form(word: &str) -> bool {
    DERIVATIONAL_PREFIXES.iter().any(|prefix| {
        let Some(rest) = word.strip_prefix(prefix) else {
            return false;
        };
        rest.chars().count() >= 5
            && (russian_dictionary().contains(rest) || is_known_russian_verb_form(rest))
    })
}

fn is_known_russian_verb_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }

    const ENDINGS: &[(&str, &[&str])] = &[
        ("айте", &["ать"]),
        ("ишься", &["иться"]),
        ("ешься", &["иться", "аться", "еться"]),
        ("ишь", &["ить", "еть"]),
        ("ай", &["ать"]),
        ("ит", &["ить", "еть"]),
        ("ает", &["ать"]),
        ("ают", &["ать"]),
        ("аешь", &["ать"]),
        ("аете", &["ать"]),
        ("ется", &["ться"]),
        ("ются", &["ться"]),
        ("ился", &["иться"]),
        ("илась", &["иться"]),
        ("ились", &["иться"]),
        ("илось", &["иться"]),
        ("ался", &["аться"]),
        ("алась", &["аться"]),
        ("ались", &["аться"]),
        ("алось", &["аться"]),
        ("ил", &["ить"]),
        ("ила", &["ить"]),
        ("или", &["ить"]),
        ("ило", &["ить"]),
        ("ал", &["ать"]),
        ("ала", &["ать"]),
        ("али", &["ать"]),
        ("ало", &["ать"]),
    ];

    ENDINGS.iter().any(|(ending, lemmas)| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        stem.chars().count() >= 3
            && lemmas
                .iter()
                .any(|lemma_suffix| russian_dictionary().contains(&format!("{stem}{lemma_suffix}")))
    })
}

fn is_known_short_accusative_a_form(word: &str, dict: &HashSet<String>) -> bool {
    let Some(stem) = word.strip_suffix('у') else {
        return false;
    };
    if stem.chars().count() < 2 {
        return false;
    }
    let lemma = format!("{stem}а");
    dict.contains(&lemma)
}

fn is_russian_consonant(ch: char) -> bool {
    is_cyrillic_letter(ch) && !is_russian_vowel(ch) && !matches!(ch, 'ь' | 'Ь' | 'ъ' | 'Ъ')
}

fn load_hunspell_words_min_len(path: &str, min_chars: usize) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for line in text.lines().skip(1) {
        let word = line.split('/').next().unwrap_or("").trim();
        if word.chars().count() >= min_chars && is_cyrillic_word(word) {
            words.insert(word.to_lowercase());
        }
    }
    Ok(words)
}

struct HunspellSuffixRule {
    strip: String,
    add: String,
    condition: Vec<HunspellConditionToken>,
}

#[derive(Clone)]
enum HunspellConditionToken {
    Literal(char),
    Class { negated: bool, chars: Vec<char> },
}

fn load_hunspell_generated_forms_min_len(
    dic_path: &str,
    aff_path: &str,
    min_chars: usize,
) -> std::io::Result<HashSet<String>> {
    let rules = load_simple_hunspell_suffix_rules(aff_path)?;
    let text = std::fs::read_to_string(dic_path)?;
    let mut forms = HashSet::new();

    for line in text.lines().skip(1) {
        let line = line.trim();
        let Some((word, flags)) = line.split_once('/') else {
            continue;
        };
        let word = word.trim().to_lowercase();
        if word.is_empty() {
            continue;
        }
        let flags = flags.split_whitespace().next().unwrap_or("");
        for flag in flags.chars() {
            let Some(flag_rules) = rules.get(&flag) else {
                continue;
            };
            for rule in flag_rules {
                if !hunspell_condition_matches(&word, &rule.condition) {
                    continue;
                }
                let stem = if rule.strip == "0" {
                    word.as_str()
                } else if let Some(stem) = word.strip_suffix(&rule.strip) {
                    stem
                } else {
                    continue;
                };
                let candidate = if rule.add == "0" {
                    stem.to_string()
                } else {
                    format!("{stem}{}", rule.add)
                };
                if candidate.chars().count() >= min_chars && is_cyrillic_word(&candidate) {
                    forms.insert(candidate);
                }
            }
        }
    }

    Ok(forms)
}

fn load_simple_hunspell_suffix_rules(
    path: &str,
) -> std::io::Result<HashMap<char, Vec<HunspellSuffixRule>>> {
    let text = std::fs::read_to_string(path)?;
    let mut rules: HashMap<char, Vec<HunspellSuffixRule>> = HashMap::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 || parts[0] != "SFX" || parts[3].parse::<usize>().is_ok() {
            continue;
        }
        let Some(flag) = parts[1].chars().next() else {
            continue;
        };
        let Some(condition) = parse_hunspell_suffix_condition(parts[4]) else {
            continue;
        };
        rules.entry(flag).or_default().push(HunspellSuffixRule {
            strip: parts[2].to_string(),
            add: parts[3].split('/').next().unwrap_or(parts[3]).to_string(),
            condition,
        });
    }

    Ok(rules)
}

fn parse_hunspell_suffix_condition(condition: &str) -> Option<Vec<HunspellConditionToken>> {
    if condition == "." {
        return Some(Vec::new());
    }

    let mut tokens = Vec::new();
    let mut chars = condition.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            let negated = if chars.peek() == Some(&'^') {
                chars.next();
                true
            } else {
                false
            };
            let mut class_chars = Vec::new();
            let mut closed = false;
            for class_ch in chars.by_ref() {
                if class_ch == ']' {
                    closed = true;
                    break;
                }
                if !is_cyrillic_letter(class_ch) {
                    return None;
                }
                class_chars.push(class_ch);
            }
            if !closed || class_chars.is_empty() {
                return None;
            }
            tokens.push(HunspellConditionToken::Class {
                negated,
                chars: class_chars,
            });
        } else if is_cyrillic_letter(ch) {
            tokens.push(HunspellConditionToken::Literal(ch));
        } else {
            return None;
        }
    }

    (!tokens.is_empty()).then_some(tokens)
}

fn hunspell_condition_matches(word: &str, condition: &[HunspellConditionToken]) -> bool {
    if condition.is_empty() {
        return true;
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.len() < condition.len() {
        return false;
    }
    let start = chars.len() - condition.len();
    condition
        .iter()
        .zip(chars[start..].iter().copied())
        .all(|(token, ch)| match token {
            HunspellConditionToken::Literal(expected) => *expected == ch,
            HunspellConditionToken::Class { negated, chars } => {
                let contains = chars.contains(&ch);
                if *negated {
                    !contains
                } else {
                    contains
                }
            }
        })
}

fn load_word_list(path: &std::path::Path) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_lowercase)
        .collect())
}
