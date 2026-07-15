use super::tokenize::{normalize_ngram_word, normalize_text};
use std::collections::HashMap;

const N: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    Ru,
    En,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharNgramModel {
    pub(super) lang: Lang,
    counts: HashMap<String, usize>,
    total: usize,
    pub(super) vocab: usize,
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
            let Some(word) = normalize_ngram_word(word.as_ref(), lang) else {
                continue;
            };
            for gram in char_ngrams(&word) {
                *counts.entry(gram).or_insert(0) += 1;
                total += 1;
            }
        }

        Self::from_counts(lang, counts, total)
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

        Self::from_counts(lang, counts, total)
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

    fn from_counts(lang: Lang, counts: HashMap<String, usize>, total: usize) -> Self {
        let vocab = counts.len().max(1);
        Self {
            lang,
            counts,
            total: total.max(1),
            vocab,
        }
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
