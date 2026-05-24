use crate::keyboard::is_cyrillic_letter;
use crate::russian_chars::is_russian_vowel;
use crate::russian_prefixes::DERIVATIONAL_PREFIXES;
use std::collections::HashSet;

use super::{
    russian_dictionary, russian_generated_form_dictionary, russian_short_dictionary,
    russian_tiny_dictionary,
};

pub(crate) fn is_known_russian_form(word: &str) -> bool {
    is_known_russian_suffix_form(word)
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
        if is_known_russian_adjective_form(stem, suffix) {
            return true;
        }
        if russian_dictionary().contains(stem) {
            return true;
        }
        matches!(*suffix, "ами" | "ями")
            && (russian_short_dictionary().contains(stem)
                || russian_dictionary().contains(&format!("{stem}о")))
    })
}

fn is_known_russian_adjective_form(stem: &str, suffix: &str) -> bool {
    if stem.chars().count() < 3 {
        return false;
    }
    if !matches!(
        suffix,
        "ыми"
            | "ими"
            | "ого"
            | "его"
            | "ому"
            | "ему"
            | "ом"
            | "ем"
            | "ой"
            | "ей"
            | "ая"
            | "яя"
            | "ое"
            | "ее"
            | "ые"
            | "ие"
    ) {
        return false;
    }
    ["ый", "ий", "ой"]
        .iter()
        .any(|ending| russian_dictionary().contains(&format!("{stem}{ending}")))
        || ["ов", "ев"].iter().any(|suffix| {
            let Some(noun_stem) = stem.strip_suffix(suffix) else {
                return false;
            };
            noun_stem.chars().count() >= 3
                && (russian_tiny_dictionary().contains(noun_stem)
                    || russian_dictionary().contains(noun_stem))
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
