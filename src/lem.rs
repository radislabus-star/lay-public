//! Layout Error Metric scorer for already-built correction candidates.
//!
//! LEM does not generate free text. It scores deterministic candidates from the
//! daemon and helps choose the most natural short tail.

use crate::dict::{self, Direction};
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    extend_common_ru_words, is_ru_short_function_word, EN_HUNSPELL, EN_WORDS, RU_HUNSPELL,
};
use crate::ngram;
use crate::text_metrics::{
    common_replacement_span, damerau_levenshtein, normalized_edit_distance, without_whitespace,
};
use crate::word_recognizer::{
    is_ascii_technical_or_brand_token, is_mixed_cyrillic_ascii_alpha_token,
};
use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct ScoredCandidate {
    pub text: String,
    pub total: f64,
    pub language: f64,
    pub noise: f64,
    pub edit: f64,
    pub intervention: f64,
}

pub fn rank_candidates<I>(typed: &str, candidates: I) -> Vec<ScoredCandidate>
where
    I: IntoIterator<Item = String>,
{
    let mut seen = HashSet::new();
    let mut ranked = Vec::new();
    for candidate in candidates {
        let candidate = candidate.trim().to_string();
        if candidate.is_empty() || !seen.insert(candidate.clone()) {
            continue;
        }
        ranked.push(score_candidate(typed, candidate));
    }
    ranked.sort_by(|a, b| b.total.total_cmp(&a.total));
    ranked
}

pub fn best_candidate<I>(typed: &str, candidates: I) -> Option<ScoredCandidate>
where
    I: IntoIterator<Item = String>,
{
    rank_candidates(typed, candidates).into_iter().next()
}

pub fn warm_up() {
    crate::lexicon::warm_up();
    let _ = ru_words().len();
    let _ = en_words().len();
    crate::ngram::warm_up();
}

fn score_candidate(typed: &str, candidate: String) -> ScoredCandidate {
    let language = language_score(&candidate);
    let noise = noise_cost(typed, &candidate);
    let edit = normalized_edit_distance(typed, &candidate);
    let intervention = intervention_penalty(typed, &candidate);
    let total =
        language + structure_bonus(typed, &candidate) + keep_valid_source_bonus(typed, &candidate)
            - (1.0 + noise).ln()
            - edit * 0.25
            - intervention;
    ScoredCandidate {
        text: candidate,
        total,
        language,
        noise,
        edit,
        intervention,
    }
}

fn language_score(text: &str) -> f64 {
    let mut total = 0.0;
    let mut count = 0usize;
    for token in text.split_whitespace() {
        let token = trim_token(token);
        if token.is_empty() {
            continue;
        }
        total += token_language_score(token);
        count += 1;
    }
    if count == 0 {
        -20.0
    } else {
        total / count as f64
    }
}

fn token_language_score(token: &str) -> f64 {
    let lower = token.to_lowercase();
    if is_short_russian_function_word(&lower) {
        return -5.5;
    }
    if is_mixed_cyrillic_ascii_alpha_token(token) {
        return -22.0;
    }
    if is_layout_garbage_token(token) {
        return -18.0;
    }
    if is_ascii_technical_or_brand_token(token) {
        return -5.0;
    }

    let alpha_count = token.chars().filter(|ch| ch.is_alphabetic()).count().max(1) as f64;
    let ru = ngram::ru_score(token);
    let en = ngram::en_score(token);
    let ru_norm = if ru.is_finite() {
        ru / alpha_count
    } else {
        -20.0
    };
    let en_norm = if en.is_finite() {
        en / alpha_count
    } else {
        -20.0
    };
    ru_norm.max(en_norm) + lexical_bonus(token)
}

fn noise_cost(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    if dict::convert(typed, Direction::Us2Ru) == candidate
        || dict::convert(typed, Direction::Ru2Us) == candidate
    {
        return 0.15;
    }
    if flip_tokens(typed) == candidate {
        return 0.25;
    }
    if let Some(cost) = partial_token_flip_cost(typed, candidate) {
        return cost;
    }
    if typed.replace(' ', "") == candidate.replace(' ', "") {
        return 0.35;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return 0.08;
    }

    let distance = damerau_levenshtein(typed, candidate) as f64;
    let scale = typed.chars().count().max(candidate.chars().count()).max(1) as f64;
    if distance <= 2.0 {
        return 0.10 + distance / scale * 0.25;
    }
    0.75 + distance / scale
}

fn partial_token_flip_cost(typed: &str, candidate: &str) -> Option<f64> {
    let typed_tokens: Vec<&str> = typed.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    if typed_tokens.len() != candidate_tokens.len() || typed_tokens.is_empty() {
        return None;
    }

    let mut changed = 0usize;
    for (typed_token, candidate_token) in typed_tokens.iter().zip(candidate_tokens.iter()) {
        if typed_token == candidate_token {
            continue;
        }
        if dict::convert(typed_token, dict::detect_direction(typed_token)) == *candidate_token {
            changed += 1;
            continue;
        }
        return None;
    }

    (changed > 0).then_some(0.18 + changed as f64 * 0.10)
}

fn intervention_penalty(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    let mut protected_penalty = 0.0;
    let typed_tokens: Vec<&str> = typed.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    let same_letters_with_moved_spaces = without_whitespace(typed) == without_whitespace(candidate);
    if typed_tokens.len() == candidate_tokens.len() && !same_letters_with_moved_spaces {
        for (typed_token, candidate_token) in typed_tokens.iter().zip(candidate_tokens.iter()) {
            let typed_raw = *typed_token;
            let typed_token = trim_token(typed_token);
            let candidate_token = trim_token(candidate_token);
            if typed_token != candidate_token
                && is_known_word(typed_token)
                && !has_ascii_layout_letter_punctuation(typed_raw)
            {
                protected_penalty += 2.0;
            }
        }
    }

    let touched = common_replacement_span(typed, candidate) as f64;
    let mut penalty = 0.08 + touched * 0.006 + protected_penalty;
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    if candidate_spaces < typed_spaces {
        penalty += (typed_spaces - candidate_spaces) as f64 * 6.0;
    } else if candidate_spaces > typed_spaces {
        penalty += (candidate_spaces - typed_spaces) as f64 * 0.08;
    }
    penalty
}

fn structure_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    let same_letters_with_moved_spaces = without_whitespace(typed) == without_whitespace(candidate);
    if typed_spaces == 0 && candidate_spaces > 0 && same_letters_with_moved_spaces {
        return 1.45;
    }
    if typed_spaces == candidate_spaces && same_letters_with_moved_spaces {
        return 0.30;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return 0.25;
    }

    let typed_known = typed
        .split_whitespace()
        .filter(|token| is_known_word(trim_token(token)))
        .count();
    let candidate_known = candidate
        .split_whitespace()
        .filter(|token| is_known_word(trim_token(token)))
        .count();
    if typed_spaces == candidate_spaces && candidate_known > typed_known {
        return 0.45;
    }
    0.0
}

fn keep_valid_source_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate && is_known_text(typed) {
        1.25
    } else {
        0.0
    }
}

fn removes_extra_repeated_letter(typed: &str, candidate: &str) -> bool {
    if typed == candidate || typed.chars().count() <= candidate.chars().count() {
        return false;
    }

    let typed_chars: Vec<char> = typed.chars().collect();
    for idx in 1..typed_chars.len() {
        if typed_chars[idx] != typed_chars[idx - 1] {
            continue;
        }
        let mut repaired = String::with_capacity(typed.len());
        for (pos, ch) in typed_chars.iter().enumerate() {
            if pos != idx {
                repaired.push(*ch);
            }
        }
        if repaired == candidate {
            return true;
        }
    }
    false
}

fn flip_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| dict::convert(token, dict::detect_direction(token)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn trim_token(token: &str) -> &str {
    token.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
}

fn is_short_russian_function_word(token: &str) -> bool {
    is_ru_short_function_word(token)
}

fn is_known_text(text: &str) -> bool {
    let mut saw_token = false;
    for token in text.split_whitespace().map(trim_token) {
        if token.is_empty() {
            continue;
        }
        saw_token = true;
        if !is_plausible_token(token) {
            return false;
        }
    }
    saw_token
}

fn is_plausible_token(token: &str) -> bool {
    !is_layout_garbage_token(token) && is_plausible_non_layout_token(token)
}

fn is_plausible_non_layout_token(token: &str) -> bool {
    is_known_word(token)
        || is_ascii_technical_or_brand_token(token)
        || is_natural_hyphenated_token(token)
}

fn is_natural_hyphenated_token(token: &str) -> bool {
    let mut saw_part = false;
    let mut alpha_parts = 0usize;
    let mut known_parts = 0usize;
    let mut long_parts = 0usize;
    for part in token.split('-') {
        if part.is_empty() {
            return false;
        }
        saw_part = true;
        let alpha_count = part.chars().filter(|ch| ch.is_alphabetic()).count();
        if alpha_count == 0 || alpha_count != part.chars().count() {
            return false;
        }
        if alpha_count >= 2 {
            alpha_parts += 1;
        }
        if alpha_count >= 3 {
            long_parts += 1;
        }
        if is_known_word(part) {
            known_parts += 1;
        }
    }
    saw_part && alpha_parts >= 2 && (known_parts > 0 || long_parts > 0)
}

fn lexical_bonus(token: &str) -> f64 {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return 0.0;
    }
    if is_known_word(token) {
        return 1.15;
    }
    if token.contains('-')
        && token
            .split('-')
            .filter(|part| !part.is_empty())
            .all(is_known_word)
    {
        return 0.90;
    }
    if is_natural_hyphenated_token(token) {
        return 0.20;
    }
    if token.chars().any(|ch| ch.is_alphabetic()) {
        return -0.55;
    }
    0.0
}

fn is_layout_garbage_token(token: &str) -> bool {
    if is_known_word(token)
        || (token.chars().any(is_cyrillic_letter) && is_natural_hyphenated_token(token))
    {
        return false;
    }
    if token.chars().filter(|ch| ch.is_alphabetic()).count() < 3 {
        return false;
    }
    let converted = dict::convert(token, dict::detect_direction(token));
    converted != token && is_plausible_non_layout_token(&converted)
}

fn has_ascii_layout_letter_punctuation(token: &str) -> bool {
    token
        .chars()
        .any(|ch| matches!(ch, '\'' | ';' | '[' | ']' | '`' | ',' | '.'))
}

fn is_known_word(token: &str) -> bool {
    let lower = token.to_lowercase();
    if lower.chars().all(is_cyrillic_letter) {
        return ru_words().contains(&lower) || is_known_ru_form(&lower);
    }
    if lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return en_words().contains(&lower);
    }
    false
}

fn is_known_ru_form(word: &str) -> bool {
    if word.chars().count() < 4 {
        return false;
    }
    if let Some(stem) = word.strip_suffix('о') {
        return ["ый", "ий", "ой"]
            .iter()
            .any(|suffix| ru_words().contains(&format!("{stem}{suffix}")));
    }
    for suffix in [
        "ками", "ках", "кой", "ки", "ке", "ку", "ок", "ыми", "ими", "ами", "ями", "ого", "его",
        "ому", "ему", "ов", "ев", "ей", "ах", "ях", "ам", "ям", "ом", "ем", "ой", "ый", "ий", "ая",
        "яя", "ое", "ее", "ые", "ие",
    ] {
        let Some(stem) = word.strip_suffix(suffix) else {
            continue;
        };
        let min_stem_len = if matches!(suffix, "ками" | "ках" | "кой" | "ки" | "ке" | "ку" | "ок")
        {
            3
        } else {
            4
        };
        if stem.chars().count() >= min_stem_len && ru_words().contains(stem) {
            return true;
        }
        if stem.chars().count() >= min_stem_len && ru_words().contains(&format!("{stem}ка")) {
            return true;
        }
    }
    for (ending, lemmas) in [
        ("шу", &["сать"][..]),
        ("ешь", &["ить", "еть"][..]),
        ("ишь", &["ить", "еть"]),
        ("ет", &["ить", "еть"]),
        ("ит", &["ить", "еть"]),
        ("ется", &["ться"]),
        ("ются", &["ться"]),
        ("ил", &["ить"]),
        ("ила", &["ить"]),
        ("или", &["ить"]),
        ("ал", &["ать"]),
        ("ала", &["ать"]),
        ("али", &["ать"]),
    ] {
        let Some(stem) = word.strip_suffix(ending) else {
            continue;
        };
        if lemmas
            .iter()
            .any(|lemma_suffix| ru_words().contains(&format!("{stem}{lemma_suffix}")))
        {
            return true;
        }
    }
    false
}

fn ru_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(RU_HUNSPELL, |word| {
            word.chars().count() >= 2 && word.chars().all(is_cyrillic_letter)
        });
        extend_common_ru_words(&mut words);
        words
    })
}

fn en_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, |word| {
            word.chars().count() >= 2 && word.chars().all(|ch| ch.is_ascii_alphabetic())
        });
        if let Ok(text) = std::fs::read_to_string(EN_WORDS) {
            words.extend(
                text.lines()
                    .map(str::trim)
                    .map(str::to_lowercase)
                    .filter(|word| {
                        word.chars().count() >= 2 && word.chars().all(|ch| ch.is_ascii_alphabetic())
                    }),
            );
        }
        words
    })
}

fn load_hunspell_words(path: &str, keep: fn(&str) -> bool) -> HashSet<String> {
    std::fs::read_to_string(path)
        .map(|text| {
            text.lines()
                .skip(1)
                .filter_map(|line| line.split('/').next())
                .map(str::trim)
                .map(str::to_lowercase)
                .filter(|word| keep(word))
                .collect()
        })
        .unwrap_or_default()
}
