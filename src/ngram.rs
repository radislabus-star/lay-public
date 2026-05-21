//! Lightweight char n-gram scorer for short local decisions.
//!
//! This is not a generator. It only compares ready candidates and answers:
//! which text looks more natural for the model trained from local dictionaries
//! and optional local user word lists.

use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::OnceLock;

const N: usize = 3;
const CACHE_VERSION: u32 = 1;
const RU_HUNSPELL: &str = "/usr/share/hunspell/ru_RU.dic";
const EN_HUNSPELL: &str = "/usr/share/hunspell/en_US.dic";
const EN_WORDS: &str = "/usr/share/dict/words";
const PROTECTED_WORDS_PATH: &str = ".config/lay/protected_words.txt";
const RU_CACHE_PATH: &str = ".cache/lay/ngram_ru_v1.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    Ru,
    En,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharNgramModel {
    lang: Lang,
    counts: HashMap<String, usize>,
    total: usize,
    vocab: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedModel {
    version: u32,
    model: CharNgramModel,
}

impl CharNgramModel {
    pub fn train<I, S>(lang: Lang, words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut counts = HashMap::new();
        let mut total = 0;

        for word in words {
            let Some(word) = normalize_word(word.as_ref(), lang) else {
                continue;
            };
            for gram in char_ngrams(&word) {
                *counts.entry(gram).or_insert(0) += 1;
                total += 1;
            }
        }

        let vocab = counts.len().max(1);
        Self {
            lang,
            counts,
            total: total.max(1),
            vocab,
        }
    }

    pub fn train_from_text(lang: Lang, text: &str) -> Self {
        let mut counts = HashMap::new();
        let mut total = 0;

        for line in text.lines() {
            let Some(line) = normalize_text(line, lang) else {
                continue;
            };
            for gram in char_ngrams(&line) {
                *counts.entry(gram).or_insert(0) += 1;
                total += 1;
            }
        }

        let vocab = counts.len().max(1);
        Self {
            lang,
            counts,
            total: total.max(1),
            vocab,
        }
    }

    pub fn score_text(&self, text: &str) -> f64 {
        let Some(text) = normalize_text(text, self.lang) else {
            return f64::NEG_INFINITY;
        };
        let mut sum = 0.0;
        let mut grams = 0;

        for gram in char_ngrams(&text) {
            let count = self.counts.get(&gram).copied().unwrap_or(0) + 1;
            let denom = self.total + self.vocab;
            sum += (count as f64 / denom as f64).ln();
            grams += 1;
        }

        if grams == 0 {
            f64::NEG_INFINITY
        } else {
            sum
        }
    }

    pub fn margin(&self, candidate: &str, baseline: &str) -> f64 {
        self.score_text(candidate) - self.score_text(baseline)
    }

    pub fn candidate_is_better(&self, candidate: &str, baseline: &str, min_margin: f64) -> bool {
        self.margin(candidate, baseline) >= min_margin
    }
}

pub fn tokenize_text(text: &str, lang: Lang) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphabetic() && ch != '-')
        .filter_map(|word| normalize_word(word, lang))
        .collect()
}

fn normalize_text(text: &str, lang: Lang) -> Option<String> {
    let tokens = tokenize_text(text, lang);
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

pub fn ru_score(text: &str) -> f64 {
    ru_model().score_text(text)
}

pub fn ru_candidate_margin(candidate: &str, baseline: &str) -> f64 {
    ru_model().margin(candidate, baseline)
}

pub fn ru_candidate_is_better(candidate: &str, baseline: &str, min_margin: f64) -> bool {
    ru_model().candidate_is_better(candidate, baseline, min_margin)
}

pub fn en_score(text: &str) -> f64 {
    en_model().score_text(text)
}

pub fn warm_up() {
    let _ = ru_model().vocab;
    let _ = en_model().vocab;
}

pub fn build_ru_model_from_sources() -> CharNgramModel {
    let mut words = Vec::new();
    words.extend(load_hunspell_words(RU_HUNSPELL, Lang::Ru));
    if let Some(home) = std::env::var_os("HOME") {
        let path = std::path::PathBuf::from(home).join(PROTECTED_WORDS_PATH);
        words.extend(load_plain_words(&path, Lang::Ru));
    }
    CharNgramModel::train(Lang::Ru, words)
}

pub fn default_ru_cache_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(RU_CACHE_PATH))
}

pub fn load_ru_cache(path: &std::path::Path) -> io::Result<CharNgramModel> {
    let text = std::fs::read_to_string(path)?;
    let cached: CachedModel = serde_json::from_str(&text).map_err(io::Error::other)?;
    if cached.version != CACHE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported ngram cache version {}", cached.version),
        ));
    }
    if cached.model.lang != Lang::Ru {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ngram cache language is not RU",
        ));
    }
    Ok(cached.model)
}

pub fn save_ru_cache(path: &std::path::Path, model: &CharNgramModel) -> io::Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cached = CachedModel {
        version: CACHE_VERSION,
        model: model.clone(),
    };
    let text = serde_json::to_string(&cached).map_err(io::Error::other)?;
    std::fs::write(path, text)?;
    Ok(std::fs::metadata(path)?.len())
}

fn ru_model() -> &'static CharNgramModel {
    static MODEL: OnceLock<CharNgramModel> = OnceLock::new();
    MODEL.get_or_init(|| {
        if let Some(path) = default_ru_cache_path() {
            if let Ok(model) = load_ru_cache(&path) {
                return model;
            }
        }
        let model = build_ru_model_from_sources();
        if let Some(path) = default_ru_cache_path() {
            let _ = save_ru_cache(&path, &model);
        }
        model
    })
}

fn en_model() -> &'static CharNgramModel {
    static MODEL: OnceLock<CharNgramModel> = OnceLock::new();
    MODEL.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, Lang::En);
        if words.is_empty() {
            words.extend(load_plain_words(std::path::Path::new(EN_WORDS), Lang::En));
        }
        CharNgramModel::train(Lang::En, words)
    })
}

fn load_hunspell_words(path: &str, lang: Lang) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .skip(1)
        .filter_map(|line| normalize_word(line.split('/').next().unwrap_or(""), lang))
        .collect()
}

fn load_plain_words(path: &std::path::Path, lang: Lang) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| normalize_word(line, lang))
        .collect()
}

fn normalize_word(word: &str, lang: Lang) -> Option<String> {
    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return None;
    }
    if word.chars().count() < 1 {
        return None;
    }
    if !word.chars().all(|ch| is_word_char(ch, lang)) {
        return None;
    }
    Some(word)
}

fn is_word_char(ch: char, lang: Lang) -> bool {
    match lang {
        Lang::Ru => matches!(ch, 'а'..='я' | 'ё' | '-'),
        Lang::En => ch.is_ascii_lowercase() || ch == '-',
    }
}

fn char_ngrams(word: &str) -> Vec<String> {
    let mut chars = Vec::with_capacity(word.chars().count() + N);
    chars.extend(std::iter::repeat('^').take(N - 1));
    chars.extend(word.chars());
    chars.push('$');

    chars
        .windows(N)
        .map(|window| window.iter().collect())
        .collect()
}

#[allow(dead_code)]
fn unique_chars(words: &[String]) -> HashSet<char> {
    words.iter().flat_map(|word| word.chars()).collect()
}

#[cfg(test)]
#[path = "ngram_tests.rs"]
mod tests;
