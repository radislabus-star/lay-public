use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::feedback::{FeedbackAdjustment, L3Feedback};
use super::options::WaveOptions;
use super::signal::{LayerTrace, WordCandidate};
use crate::russian_chars::is_russian_vowel;

pub const LLMWAVE_CELL: &str = "LLMWaveCell32";
pub const LLMWAVE_RECORD_BYTES: usize = 32;
pub const LLMWAVE_MAGIC: &[u8; 8] = b"LLMWAVE1";
const HEADER_BYTES: usize = 64;
const SCHEMA_ID: &str = "lay.llmwave.memory.v1";
const TOKENIZER_ID: &str = "lay.llmwave.tokenizer.v1";
const MODEL_ID: &str = "lay.llmwave.l3_shadow.v1";
const PHRASE_EXPERIENCE_PATH: &str = ".local/share/lay/nanda_wave/phrase_experience.jsonl";
const MIN_EXPERIENCE_TOKENS: usize = 3;
const MAX_EXPERIENCE_TOKENS: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct LlmWaveContract {
    pub model_id: &'static str,
    pub schema_id: &'static str,
    pub tokenizer_id: &'static str,
    pub record_bytes: usize,
    pub hot_path: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmWaveReport {
    pub records: usize,
    pub top: Option<LlmWaveCandidateScore>,
    pub candidates: Vec<LlmWaveCandidateScore>,
    pub predictions: Vec<LlmWavePhrasePrediction>,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmWaveCandidateScore {
    pub source: &'static str,
    pub text: String,
    pub score: f32,
    pub support: usize,
    pub width: usize,
    pub likelihood: f32,
    pub prior: f32,
    pub phase_coherence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmWaveNextTokenScore {
    pub score: f32,
    pub support: usize,
    pub width: usize,
    pub likelihood: f32,
    pub prior: f32,
    pub phase_coherence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmWavePhrasePrediction {
    pub text: String,
    pub score: f32,
    pub support: usize,
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmWaveLearningDelta {
    pub phrase: String,
    pub prefix: String,
    pub next_token: String,
    pub seed_score: f32,
    pub live_score: f32,
    pub live_support: usize,
    pub width: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmWavePhraseExperience {
    pub kind: String,
    pub ts: u64,
    pub stage: String,
    pub text: String,
    pub tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhraseExperienceRejectReason {
    UnsupportedStage,
    UnsafeText,
    TokenCount,
    DirtyLayout,
    LowLanguageQuality,
    UnstableOrNoisy,
}

impl PhraseExperienceRejectReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedStage => "unsupported_stage",
            Self::UnsafeText => "unsafe_text",
            Self::TokenCount => "token_count",
            Self::DirtyLayout => "dirty_layout",
            Self::LowLanguageQuality => "low_language_quality",
            Self::UnstableOrNoisy => "unstable_or_noisy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlmWaveRecord {
    pub prefix_hash: u32,
    pub token_hash: u32,
    pub next_hash: u32,
    pub route_hash: u32,
    pub strength: i16,
    pub accepted: u16,
    pub rejected: u16,
    pub flags: u16,
    pub phase: i8,
    pub lens: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmWaveMemory {
    records: Vec<LlmWaveRecord>,
    vocabulary: BTreeMap<u32, String>,
}

impl LlmWaveMemory {
    pub fn empty() -> Self {
        Self {
            records: Vec::new(),
            vocabulary: BTreeMap::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        let mut records = BTreeMap::<(u32, u32, u32), LlmWaveRecord>::new();
        let mut vocabulary = BTreeMap::new();
        for line in text.lines() {
            let tokens = tokenize(line);
            if tokens.len() < 2 {
                continue;
            }
            let line_memory = Self::from_token_stream(&tokens);
            vocabulary.extend(line_memory.vocabulary);
            for record in line_memory.records {
                let key = (record.prefix_hash, record.token_hash, record.next_hash);
                records
                    .entry(key)
                    .and_modify(|entry| {
                        entry.strength = entry.strength.saturating_add(record.strength.max(0));
                        entry.accepted = entry.accepted.saturating_add(record.accepted);
                        entry.rejected = entry.rejected.saturating_add(record.rejected);
                    })
                    .or_insert(record);
            }
        }
        Self {
            records: records.into_values().collect(),
            vocabulary,
        }
    }

    pub fn from_token_stream(tokens: &[String]) -> Self {
        let mut records = BTreeMap::<(u32, u32, u32), LlmWaveRecord>::new();
        let vocabulary = tokens
            .iter()
            .map(|token| (token_hash(token), token.clone()))
            .collect::<BTreeMap<_, _>>();
        for idx in 1..tokens.len() {
            for width in 1..=3.min(idx) {
                let start = idx - width;
                let prefix = prefix_hash(&tokens[start..idx]);
                let token = token_hash(&tokens[idx - 1]);
                let next = token_hash(&tokens[idx]);
                let route = route_hash_for(&tokens[start..=idx]);
                let key = (prefix, token, next);
                let entry = records.entry(key).or_insert(LlmWaveRecord {
                    prefix_hash: prefix,
                    token_hash: token,
                    next_hash: next,
                    route_hash: route,
                    strength: 0,
                    accepted: 0,
                    rejected: 0,
                    flags: 0,
                    phase: phase_for(prefix, token, next),
                    lens: width as u8,
                });
                entry.strength = entry.strength.saturating_add(256);
                entry.accepted = entry.accepted.saturating_add(1);
            }
        }
        Self {
            records: records.into_values().collect(),
            vocabulary,
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> &[LlmWaveRecord] {
        &self.records
    }

    pub fn vocabulary_len(&self) -> usize {
        self.vocabulary.len()
    }

    pub fn score_next_token(&self, prefix_tokens: &[String], next_token: &str) -> (f32, usize) {
        self.score_next_token_report(prefix_tokens, next_token)
            .map(|score| (score.score, score.support))
            .unwrap_or((0.0, 0))
    }

    pub fn score_next_token_report(
        &self,
        prefix_tokens: &[String],
        next_token: &str,
    ) -> Option<LlmWaveNextTokenScore> {
        if prefix_tokens.is_empty() {
            return None;
        }
        let token = token_hash(prefix_tokens.last().map(String::as_str).unwrap_or_default());
        let next = token_hash(next_token);
        let prior = self.next_token_prior(next);
        for width in (1..=3.min(prefix_tokens.len())).rev() {
            let start = prefix_tokens.len() - width;
            let prefix = prefix_hash(&prefix_tokens[start..]);
            let mut total_energy = 0.0_f32;
            let mut hit_energy = 0.0_f32;
            let mut support = 0_usize;
            let mut phase_sum = 0.0_f32;
            for record in self
                .records
                .iter()
                .filter(|record| record.prefix_hash == prefix && record.token_hash == token)
            {
                let energy = record_energy(record);
                total_energy += energy;
                if record.next_hash == next {
                    hit_energy += energy;
                    support += 1;
                    phase_sum += phase_coherence(record.phase);
                }
            }
            if support == 0 || total_energy <= f32::EPSILON {
                continue;
            }
            let likelihood = (hit_energy / total_energy).clamp(0.0, 1.0);
            let phase_coherence = (phase_sum / support as f32).clamp(0.0, 1.0);
            let width_confidence = width as f32 / 3.0;
            let score = (likelihood * 0.64
                + prior * 0.16
                + phase_coherence * 0.10
                + width_confidence * 0.10)
                .clamp(0.0, 1.0);
            return Some(LlmWaveNextTokenScore {
                score,
                support,
                width,
                likelihood,
                prior,
                phase_coherence,
            });
        }
        None
    }

    fn next_token_prior(&self, next: u32) -> f32 {
        let mut next_energy = 0.0_f32;
        let mut total_energy = 0.0_f32;
        for record in &self.records {
            let energy = record_energy(record);
            total_energy += energy;
            if record.next_hash == next {
                next_energy += energy;
            }
        }
        if total_energy <= f32::EPSILON {
            return 0.0;
        }
        (next_energy / total_energy).clamp(0.0, 1.0)
    }

    pub fn predict_phrase(
        &self,
        prefix_text: &str,
        steps: usize,
        beam_width: usize,
    ) -> Vec<LlmWavePhrasePrediction> {
        let tokens = tokenize(prefix_text);
        if tokens.is_empty() || self.records.is_empty() || self.vocabulary.is_empty() {
            return Vec::new();
        }
        let mut beam = vec![LlmWavePhrasePrediction {
            text: String::new(),
            score: 1.0,
            support: 0,
            tokens,
        }];
        for _ in 0..steps {
            let mut next_beam = Vec::new();
            for item in &beam {
                for next in self.next_token_predictions(&item.tokens, beam_width) {
                    let mut tokens = item.tokens.clone();
                    tokens.push(next.text.clone());
                    let score = combined_phrase_score(item, &next);
                    next_beam.push(LlmWavePhrasePrediction {
                        text: tokens.join(" "),
                        score,
                        support: item.support + next.support,
                        tokens,
                    });
                }
            }
            if next_beam.is_empty() {
                break;
            }
            next_beam.sort_by(|left, right| {
                right
                    .score
                    .total_cmp(&left.score)
                    .then_with(|| right.support.cmp(&left.support))
                    .then_with(|| left.text.cmp(&right.text))
            });
            next_beam.truncate(beam_width);
            beam = next_beam;
        }
        beam.into_iter()
            .filter(|item| item.tokens.len() > tokenize(prefix_text).len())
            .collect()
    }

    fn next_token_predictions(
        &self,
        prefix_tokens: &[String],
        limit: usize,
    ) -> Vec<LlmWavePhrasePrediction> {
        for width in (1..=3.min(prefix_tokens.len())).rev() {
            let start = prefix_tokens.len() - width;
            let prefix = prefix_hash(&prefix_tokens[start..]);
            let token = token_hash(prefix_tokens.last().map(String::as_str).unwrap_or_default());
            let mut by_hash = BTreeMap::<u32, (f32, usize)>::new();
            for record in self
                .records
                .iter()
                .filter(|record| record.prefix_hash == prefix && record.token_hash == token)
            {
                let accepted = f32::from(record.accepted);
                let rejected = f32::from(record.rejected);
                let trust = (accepted + 1.0) / (accepted + rejected + 2.0);
                let score = (f32::from(record.strength.max(0)) / 1024.0).clamp(0.0, 1.0) * trust;
                let entry = by_hash.entry(record.next_hash).or_default();
                entry.0 += score;
                entry.1 += 1;
            }
            let predictions = ranked_predictions(&self.vocabulary, by_hash, limit);
            if !predictions.is_empty() {
                return predictions;
            }
        }
        Vec::new()
    }
}

pub fn learning_deltas<'a>(
    seed_memory: &LlmWaveMemory,
    combined_memory: &LlmWaveMemory,
    live_phrases: impl Iterator<Item = &'a str>,
    limit: usize,
) -> Vec<LlmWaveLearningDelta> {
    let mut deltas = Vec::new();
    for phrase in live_phrases {
        if let Some(delta) = strongest_learning_delta(seed_memory, combined_memory, phrase) {
            deltas.push(delta);
        }
    }
    deltas.sort_by(|left, right| {
        (right.live_score - right.seed_score)
            .total_cmp(&(left.live_score - left.seed_score))
            .then_with(|| right.live_support.cmp(&left.live_support))
            .then_with(|| left.phrase.cmp(&right.phrase))
    });
    deltas.truncate(limit);
    deltas
}

fn strongest_learning_delta(
    seed_memory: &LlmWaveMemory,
    combined_memory: &LlmWaveMemory,
    phrase: &str,
) -> Option<LlmWaveLearningDelta> {
    let tokens = tokenize(phrase);
    if tokens.len() < 3 {
        return None;
    }
    let mut best = None::<LlmWaveLearningDelta>;
    for idx in 1..tokens.len() {
        let prefix = &tokens[..idx];
        let next = &tokens[idx];
        let live = combined_memory.score_next_token_report(prefix, next)?;
        let seed_score = seed_memory
            .score_next_token_report(prefix, next)
            .map(|score| score.score)
            .unwrap_or(0.0);
        if live.score <= seed_score + 0.03 || live.support == 0 {
            continue;
        }
        let delta = LlmWaveLearningDelta {
            phrase: phrase.to_string(),
            prefix: prefix.join(" "),
            next_token: next.clone(),
            seed_score,
            live_score: live.score,
            live_support: live.support,
            width: live.width,
        };
        let should_replace = match best.as_ref() {
            None => true,
            Some(current) => {
                let delta_gain = delta.live_score - delta.seed_score;
                let current_gain = current.live_score - current.seed_score;
                delta_gain > current_gain
                    || (delta_gain == current_gain && delta.live_support > current.live_support)
            }
        };
        if should_replace {
            best = Some(delta);
        }
    }
    best
}

fn record_energy(record: &LlmWaveRecord) -> f32 {
    let accepted = f32::from(record.accepted);
    let rejected = f32::from(record.rejected);
    let trust = (accepted + 1.0) / (accepted + rejected + 2.0);
    (f32::from(record.strength.max(0)) / 1024.0).clamp(0.0, 1.0) * trust
}

fn phase_coherence(phase: i8) -> f32 {
    1.0 - (phase.unsigned_abs() as f32 / 128.0).clamp(0.0, 1.0)
}

fn combined_phrase_score(current: &LlmWavePhrasePrediction, next: &LlmWavePhrasePrediction) -> f32 {
    if current.support == 0 {
        return next.score;
    }
    let current_weight = current.support.max(1) as f32;
    let next_weight = next.support.max(1) as f32;
    ((current.score * current_weight + next.score * next_weight) / (current_weight + next_weight))
        .clamp(0.0, 1.0)
}

fn ranked_predictions(
    vocabulary: &BTreeMap<u32, String>,
    by_hash: BTreeMap<u32, (f32, usize)>,
    limit: usize,
) -> Vec<LlmWavePhrasePrediction> {
    let mut predictions = by_hash
        .into_iter()
        .filter_map(|(hash, (score, support))| {
            vocabulary.get(&hash).map(|token| LlmWavePhrasePrediction {
                text: token.clone(),
                score: score.clamp(0.0, 1.0),
                support,
                tokens: vec![token.clone()],
            })
        })
        .collect::<Vec<_>>();
    predictions.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| right.support.cmp(&left.support))
            .then_with(|| left.text.cmp(&right.text))
    });
    predictions.truncate(limit);
    predictions
}

pub fn contract() -> LlmWaveContract {
    LlmWaveContract {
        model_id: MODEL_ID,
        schema_id: SCHEMA_ID,
        tokenizer_id: TOKENIZER_ID,
        record_bytes: LLMWAVE_RECORD_BYTES,
        hot_path: "L3 shadow scorer; no text output",
    }
}

pub fn default_memory_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_LLMWAVE_MEMORY").map(PathBuf::from) {
        return Some(path);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".cache/lay/llmwave/phrase_memory.llmw.bin"))
}

pub fn default_phrase_experience_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_LLMWAVE_EXPERIENCE").map(PathBuf::from) {
        return Some(path);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(PHRASE_EXPERIENCE_PATH))
}

pub fn load_default_memory() -> LlmWaveMemory {
    static MEMORY: OnceLock<LlmWaveMemory> = OnceLock::new();
    MEMORY.get_or_init(load_default_memory_uncached).clone()
}

pub fn with_default_memory<T>(f: impl FnOnce(&LlmWaveMemory) -> T) -> T {
    static MEMORY: OnceLock<LlmWaveMemory> = OnceLock::new();
    f(MEMORY.get_or_init(load_default_memory_uncached))
}

pub fn load_default_memory_uncached() -> LlmWaveMemory {
    default_memory_path()
        .and_then(|path| read_memory_packet(&path).ok())
        .unwrap_or_else(LlmWaveMemory::empty)
}

pub fn write_memory_packet(path: &Path, memory: &LlmWaveMemory) -> io::Result<()> {
    crate::private_file::write_private_bytes(path, &encode_memory(memory))
}

pub fn read_memory_packet(path: &Path) -> io::Result<LlmWaveMemory> {
    decode_memory(&fs::read(path)?)
}

pub fn record_phrase_experience(stage: &str, text: &str) {
    let Some(experience) = phrase_experience(stage, text) else {
        return;
    };
    let Some(path) = default_phrase_experience_path() else {
        return;
    };
    if let Ok(line) = serde_json::to_string(&experience) {
        crate::debug_log::append_private_line(path, line);
    }
}

pub fn load_phrase_experience_text(path: &Path) -> io::Result<String> {
    Ok(load_phrase_experience(path)?
        .into_iter()
        .map(|record| record.text)
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn load_phrase_experience(path: &Path) -> io::Result<Vec<LlmWavePhraseExperience>> {
    let text = fs::read_to_string(path)?;
    let mut records = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(record) = serde_json::from_str::<LlmWavePhraseExperience>(line) else {
            continue;
        };
        if phrase_experience(&record.stage, &record.text).is_some() {
            records.push(record);
        }
    }
    Ok(records)
}

pub fn phrase_experience_rejection_reason(
    stage: &str,
    text: &str,
) -> Option<PhraseExperienceRejectReason> {
    build_phrase_experience(stage, text).err()
}

fn phrase_experience(stage: &str, text: &str) -> Option<LlmWavePhraseExperience> {
    build_phrase_experience(stage, text).ok()
}

fn build_phrase_experience(
    stage: &str,
    text: &str,
) -> Result<LlmWavePhraseExperience, PhraseExperienceRejectReason> {
    if stage != "space" && stage != "enter" {
        return Err(PhraseExperienceRejectReason::UnsupportedStage);
    }
    let normalized = normalize_experience_text(text)?;
    let tokens = tokenize(&normalized);
    if !(MIN_EXPERIENCE_TOKENS..=MAX_EXPERIENCE_TOKENS).contains(&tokens.len()) {
        return Err(PhraseExperienceRejectReason::TokenCount);
    }
    if has_dirty_layout_fragment(&normalized, &tokens) {
        return Err(PhraseExperienceRejectReason::DirtyLayout);
    }
    if is_unstable_or_noisy_phrase(&normalized) {
        return Err(PhraseExperienceRejectReason::UnstableOrNoisy);
    }
    if has_low_language_quality(&normalized, &tokens) {
        return Err(PhraseExperienceRejectReason::LowLanguageQuality);
    }
    Ok(LlmWavePhraseExperience {
        kind: "llmwave_phrase_experience_v1".to_string(),
        ts: unix_now(),
        stage: stage.to_string(),
        text: normalized,
        tokens: tokens.len(),
    })
}

fn normalize_experience_text(text: &str) -> Result<String, PhraseExperienceRejectReason> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty()
        || text.contains("://")
        || text.contains('@')
        || text.contains('=')
        || text.chars().any(|ch| ch.is_control())
        || text
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_punctuation() || matches!(ch, '…'))
        || text
            .split_whitespace()
            .any(|token| token.starts_with('-') && token.chars().count() > 1)
    {
        return Err(PhraseExperienceRejectReason::UnsafeText);
    }
    let alpha = text.chars().filter(|ch| ch.is_alphabetic()).count();
    let total = text.chars().filter(|ch| !ch.is_whitespace()).count().max(1);
    if alpha * 2 < total {
        return Err(PhraseExperienceRejectReason::LowLanguageQuality);
    }
    Ok(text)
}

fn has_dirty_layout_fragment(text: &str, tokens: &[String]) -> bool {
    let has_cyrillic = text.chars().any(is_cyrillic_letter);
    let dirty_tokens = tokens
        .iter()
        .filter(|token| looks_like_layout_garbage_token(token))
        .count();
    (has_cyrillic && dirty_tokens > 0) || dirty_tokens >= 2
}

fn looks_like_layout_garbage_token(token: &str) -> bool {
    if token.chars().count() < 5 || !token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    let lower = token.to_ascii_lowercase();
    if crate::lexicon::is_common_en_technical_word(&lower)
        || crate::word_recognizer::is_ascii_technical_or_brand_token(token)
    {
        return false;
    }
    let converted = crate::dict::convert(token, crate::dict::Direction::Us2Ru);
    converted != token
        && converted
            .chars()
            .all(|ch| is_cyrillic_letter(ch) || ch == 'ё')
        && converted.chars().filter(|ch| is_russian_vowel(*ch)).count() >= 2
}

fn has_low_language_quality(text: &str, tokens: &[String]) -> bool {
    let alpha = text.chars().filter(|ch| ch.is_alphabetic()).count();
    let cyrillic = text.chars().filter(|ch| is_cyrillic_letter(*ch)).count();
    let ascii = text.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    if cyrillic > 0 && ascii > 0 && ascii > cyrillic * 2 {
        return true;
    }
    alpha == 0
        || ends_with_unfinished_short_word(tokens)
        || tokens.iter().any(|token| has_repeated_letter_run(token, 4))
        || tokens
            .iter()
            .any(|token| looks_like_untrusted_live_learning_token(token))
        || tokens
            .iter()
            .any(|token| looks_like_unstable_unknown_russian_token(token))
}

fn is_unstable_or_noisy_phrase(text: &str) -> bool {
    let mut prev = '\0';
    let mut run = 0_usize;
    for ch in text.chars() {
        if matches!(ch, '!' | '?' | '.' | ',' | ';' | ':') && ch == prev {
            run += 1;
            if run >= 3 {
                return true;
            }
        } else {
            prev = ch;
            run = 1;
        }
    }
    let shouting_tokens = text
        .split_whitespace()
        .filter(|token| {
            let letters = token.chars().filter(|ch| ch.is_alphabetic()).count();
            letters >= 2 && token.chars().filter(|ch| ch.is_uppercase()).count() == letters
        })
        .count();
    shouting_tokens >= 2
        || text.split_whitespace().any(|token| {
            let letters = token.chars().filter(|ch| ch.is_alphabetic()).count();
            letters >= 3 && token.chars().filter(|ch| ch.is_uppercase()).count() == letters
        })
}

fn has_repeated_letter_run(token: &str, limit: usize) -> bool {
    let mut prev = '\0';
    let mut run = 0_usize;
    for ch in token.chars() {
        if ch == prev {
            run += 1;
            if run >= limit {
                return true;
            }
        } else {
            prev = ch;
            run = 1;
        }
    }
    false
}

fn looks_like_untrusted_live_learning_token(token: &str) -> bool {
    let lower = token.to_lowercase();
    if lower.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    if lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return !(crate::lexicon::is_common_en_technical_word(&lower)
            || crate::word_recognizer::is_ascii_technical_or_brand_token(&lower));
    }
    if lower.contains('-') {
        return lower
            .split('-')
            .filter(|part| !part.is_empty())
            .any(looks_like_untrusted_live_learning_token);
    }
    if !lower.chars().all(is_cyrillic_letter) {
        return true;
    }
    let len = lower.chars().count();
    if is_known_russian_learning_token(&lower) {
        if crate::lexicon::is_common_ru_word(&lower)
            || (len <= 3 && crate::phrase_lexicon::is_known_russian_phrase_part(&lower))
            || looks_like_short_russian_imperative(&lower)
        {
            return false;
        }
        return len <= 5
            && (crate::ru_typo::has_plausible_russian_typo_candidate(&lower)
                || has_short_known_insertion_neighbor(&lower));
    }
    len <= 5 || crate::ru_typo::has_plausible_russian_typo_candidate(&lower)
}

fn looks_like_short_russian_imperative(token: &str) -> bool {
    let len = token.chars().count();
    (4..=5).contains(&len)
        && ["ай", "ей", "уй"]
            .iter()
            .any(|ending| token.ends_with(ending))
}

fn has_short_known_insertion_neighbor(token: &str) -> bool {
    let len = token.chars().count();
    if !(3..=5).contains(&len) {
        return false;
    }
    let chars = token.chars().collect::<Vec<_>>();
    let insertions = [
        'б', 'в', 'г', 'д', 'ж', 'з', 'к', 'л', 'м', 'н', 'п', 'р', 'с', 'т', 'ф', 'х', 'ц', 'ч',
        'ш', 'щ',
    ];
    for idx in 0..=chars.len() {
        for inserted in insertions {
            let mut candidate = String::with_capacity(token.len() + inserted.len_utf8());
            for (char_idx, ch) in chars.iter().enumerate() {
                if char_idx == idx {
                    candidate.push(inserted);
                }
                candidate.push(*ch);
            }
            if idx == chars.len() {
                candidate.push(inserted);
            }
            if candidate != token
                && crate::russian_lexicon::is_known_russian_word_or_form(&candidate)
            {
                return true;
            }
        }
    }
    false
}

fn looks_like_unstable_unknown_russian_token(token: &str) -> bool {
    if token.chars().count() < 6 || !token.chars().all(is_cyrillic_letter) {
        return false;
    }
    if is_known_russian_learning_token(token) {
        return false;
    }
    has_unstable_russian_vowel_bridge(token)
        || has_repeated_vowel(token)
        || looks_like_single_letter_glue_to_known_word(token)
        || crate::ru_typo::has_plausible_russian_typo_candidate(&token.to_lowercase())
}

fn is_known_russian_learning_token(token: &str) -> bool {
    let lower = token.to_lowercase();
    crate::lexicon::is_common_ru_word(&lower)
        || crate::russian_lexicon::russian_tiny_dictionary().contains(&lower)
        || crate::russian_lexicon::is_known_russian_word_or_form(&lower)
}

fn ends_with_unfinished_short_word(tokens: &[String]) -> bool {
    tokens.last().is_some_and(|token| {
        let len = token.chars().count();
        (1..=2).contains(&len) && token.chars().all(is_cyrillic_letter)
    })
}

fn looks_like_single_letter_glue_to_known_word(token: &str) -> bool {
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_cyrillic_letter(first) {
        return false;
    }
    let rest = chars.collect::<String>();
    rest.chars().count() >= 4 && is_known_russian_learning_token(&rest)
}

fn has_unstable_russian_vowel_bridge(token: &str) -> bool {
    let lower = token.to_lowercase();
    ["аей", "еей", "оей", "уей", "ыей", "эей", "яей", "уюю"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn has_repeated_vowel(token: &str) -> bool {
    let mut prev = '\0';
    for ch in token.chars().flat_map(char::to_lowercase) {
        if ch == prev && is_russian_vowel(ch) {
            return true;
        }
        prev = ch;
    }
    false
}

fn is_cyrillic_letter(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё')
}

pub fn encode_memory(memory: &LlmWaveMemory) -> Vec<u8> {
    let vocab_bytes = encode_vocabulary(&memory.vocabulary);
    let mut bytes =
        vec![0_u8; HEADER_BYTES + memory.records.len() * LLMWAVE_RECORD_BYTES + vocab_bytes.len()];
    bytes[..LLMWAVE_MAGIC.len()].copy_from_slice(LLMWAVE_MAGIC);
    bytes[8..10].copy_from_slice(&(1_u16).to_le_bytes());
    bytes[10..12].copy_from_slice(&(LLMWAVE_RECORD_BYTES as u16).to_le_bytes());
    bytes[12..16].copy_from_slice(&(memory.records.len() as u32).to_le_bytes());
    bytes[16..20].copy_from_slice(&hash32(SCHEMA_ID).to_le_bytes());
    bytes[20..24].copy_from_slice(&hash32(TOKENIZER_ID).to_le_bytes());
    bytes[24..28].copy_from_slice(&(vocab_bytes.len() as u32).to_le_bytes());
    for (idx, record) in memory.records.iter().enumerate() {
        encode_record(
            &mut bytes[HEADER_BYTES + idx * LLMWAVE_RECORD_BYTES
                ..HEADER_BYTES + (idx + 1) * LLMWAVE_RECORD_BYTES],
            *record,
        );
    }
    let vocab_start = HEADER_BYTES + memory.records.len() * LLMWAVE_RECORD_BYTES;
    bytes[vocab_start..].copy_from_slice(&vocab_bytes);
    bytes
}

pub fn decode_memory(bytes: &[u8]) -> io::Result<LlmWaveMemory> {
    if bytes.len() < HEADER_BYTES || &bytes[..LLMWAVE_MAGIC.len()] != LLMWAVE_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid LLMWave memory packet",
        ));
    }
    let record_bytes = u16::from_le_bytes([bytes[10], bytes[11]]) as usize;
    let records = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    let vocab_bytes = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]) as usize;
    let records_end = HEADER_BYTES + records * LLMWAVE_RECORD_BYTES;
    if record_bytes != LLMWAVE_RECORD_BYTES || bytes.len() != records_end + vocab_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported LLMWave memory layout",
        ));
    }
    let mut decoded = Vec::with_capacity(records);
    for idx in 0..records {
        decoded.push(decode_record(
            &bytes[HEADER_BYTES + idx * LLMWAVE_RECORD_BYTES
                ..HEADER_BYTES + (idx + 1) * LLMWAVE_RECORD_BYTES],
        ));
    }
    let vocabulary = if vocab_bytes == 0 {
        BTreeMap::new()
    } else {
        decode_vocabulary(&bytes[records_end..records_end + vocab_bytes])?
    };
    Ok(LlmWaveMemory {
        records: decoded,
        vocabulary,
    })
}

pub fn derive_llmwave_feedback(
    original: &str,
    candidates: &[WordCandidate],
    options: &WaveOptions,
) -> (Vec<LayerTrace>, L3Feedback) {
    if !options.llmwave_shadow() && !options.llmwave_apply() {
        return (Vec::new(), L3Feedback::default());
    }
    let memory = load_default_memory();
    let report = score_candidates(original, candidates, &memory);
    let trace = LayerTrace {
        name: LLMWAVE_CELL,
        summary: report_summary(&report, options),
    };
    let feedback = if options.llmwave_apply() {
        report_to_feedback(&report)
    } else {
        L3Feedback::default()
    };
    (vec![trace], feedback)
}

pub fn score_candidates(
    original: &str,
    candidates: &[WordCandidate],
    memory: &LlmWaveMemory,
) -> LlmWaveReport {
    if memory.is_empty() {
        return LlmWaveReport {
            records: memory.len(),
            top: None,
            candidates: Vec::new(),
            predictions: memory.predict_phrase(original, 3, 3),
            source: "memory",
        };
    }
    let original_tokens = tokenize(original);
    let mut scored = candidates
        .iter()
        .filter_map(|candidate| {
            let tokens = tokenize(&candidate.text);
            let (prefix, next) = candidate_prefix_and_next(&original_tokens, &tokens)?;
            let report = memory.score_next_token_report(&prefix, &next)?;
            (report.score > 0.0).then(|| LlmWaveCandidateScore {
                source: candidate.source,
                text: candidate.text.clone(),
                score: report.score,
                support: report.support,
                width: report.width,
                likelihood: report.likelihood,
                prior: report.prior,
                phase_coherence: report.phase_coherence,
            })
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| right.support.cmp(&left.support))
            .then_with(|| left.text.cmp(&right.text))
    });
    let top = scored.first().cloned();
    LlmWaveReport {
        records: memory.len(),
        top,
        candidates: scored,
        predictions: memory.predict_phrase(original, 3, 3),
        source: "memory",
    }
}

pub fn phrase_forecast_candidates(original: &str, memory: &LlmWaveMemory) -> Vec<WordCandidate> {
    if memory.is_empty() {
        return Vec::new();
    }
    let request = PhraseForecastRequest::from_text(original);
    if request.prefix_tokens.is_empty() {
        return Vec::new();
    }
    let (steps, beam_width, take) = if request.partial_token.is_empty() {
        (6, 8, 4)
    } else {
        (1, 4, 2)
    };
    memory
        .predict_phrase(&request.prefix_text, steps, beam_width)
        .into_iter()
        .filter_map(|prediction| phrase_prediction_to_candidate(original, &request, prediction))
        .take(take)
        .collect()
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let token = token
                .trim_matches(|ch: char| {
                    ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”' | '„' | '…')
                })
                .to_lowercase();
            (!token.is_empty()).then_some(token)
        })
        .collect()
}

struct PhraseForecastRequest {
    prefix_text: String,
    prefix_tokens: Vec<String>,
    partial_token: String,
}

impl PhraseForecastRequest {
    fn from_text(text: &str) -> Self {
        let trimmed = text.trim_end();
        if text.ends_with(char::is_whitespace) {
            return Self {
                prefix_text: trimmed.to_string(),
                prefix_tokens: tokenize(trimmed),
                partial_token: String::new(),
            };
        }
        let Some((prefix, token)) = split_last_token(trimmed) else {
            return Self {
                prefix_text: trimmed.to_string(),
                prefix_tokens: tokenize(trimmed),
                partial_token: String::new(),
            };
        };
        let prefix_text = prefix.trim_end().to_string();
        Self {
            prefix_tokens: tokenize(&prefix_text),
            prefix_text,
            partial_token: token.to_lowercase(),
        }
    }
}

fn phrase_prediction_to_candidate(
    original: &str,
    request: &PhraseForecastRequest,
    prediction: LlmWavePhrasePrediction,
) -> Option<WordCandidate> {
    if prediction.tokens.len() <= request.prefix_tokens.len() {
        return None;
    }
    let next_token = prediction.tokens.get(request.prefix_tokens.len())?;
    if !request.partial_token.is_empty() && !next_token.starts_with(&request.partial_token) {
        return None;
    }
    let text = prediction.text.trim().to_string();
    if text == original.trim_end() || text.chars().count() <= original.trim_end().chars().count() {
        return None;
    }
    let token_count = prediction
        .tokens
        .len()
        .saturating_sub(request.prefix_tokens.len());
    let completion_shape = if request.partial_token.is_empty() {
        "next-word"
    } else {
        "partial-token"
    };
    Some(WordCandidate {
        text,
        source: super::context_wave::PHRASE_FORECAST_CELL,
        energy: (0.52 + prediction.score * 0.34).clamp(0.0, 0.88),
        risk: (0.28 - prediction.score * 0.08).clamp(0.16, 0.28),
        support: vec![
            "llmwave-phrase-forecast".to_string(),
            format!(
                "shape={completion_shape} tokens={token_count} support={} original_len={}",
                prediction.support,
                original.chars().count()
            ),
        ],
    })
}

fn split_last_token(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return None;
    }
    let start = trimmed
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_whitespace().then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    Some((&trimmed[..start], &trimmed[start..]))
}

fn report_to_feedback(report: &LlmWaveReport) -> L3Feedback {
    let Some(top) = report.top.as_ref() else {
        return L3Feedback::default();
    };
    if top.score < 0.35 {
        return L3Feedback::default();
    }
    L3Feedback {
        adjustments: vec![FeedbackAdjustment {
            source: top.source,
            energy_delta: (top.score * 0.08).clamp(0.02, 0.08),
            risk_delta: -0.02,
            reason: "llmwave_context_memory",
        }],
        requests: Vec::new(),
    }
}

fn report_summary(report: &LlmWaveReport, options: &WaveOptions) -> String {
    let mode = if options.llmwave_apply() {
        "apply-feedback"
    } else {
        "shadow"
    };
    let prediction = report
        .predictions
        .first()
        .map(|item| format!(" predict={:?} p={:.3}", item.text, item.score))
        .unwrap_or_default();
    match report.top.as_ref() {
        Some(top) => format!(
            "{mode} records={} top_source={} score={:.3} width={} likelihood={:.3} prior={:.3} phase={:.3} support={} candidate={:?}{prediction}",
            report.records,
            top.source,
            top.score,
            top.width,
            top.likelihood,
            top.prior,
            top.phase_coherence,
            top.support,
            top.text
        ),
        None => format!("{mode} records={} top=none{prediction}", report.records),
    }
}

fn candidate_prefix_and_next(
    original_tokens: &[String],
    candidate_tokens: &[String],
) -> Option<(Vec<String>, String)> {
    if candidate_tokens.is_empty() {
        return None;
    }
    for (idx, token) in candidate_tokens.iter().enumerate() {
        if original_tokens.get(idx) != Some(token) {
            if idx == 0 {
                return None;
            }
            return Some((candidate_tokens[..idx].to_vec(), token.clone()));
        }
    }
    let idx = candidate_tokens.len().saturating_sub(1);
    (idx > 0).then(|| {
        (
            candidate_tokens[..idx].to_vec(),
            candidate_tokens[idx].clone(),
        )
    })
}

fn encode_record(slot: &mut [u8], record: LlmWaveRecord) {
    slot[0..4].copy_from_slice(&record.prefix_hash.to_le_bytes());
    slot[4..8].copy_from_slice(&record.token_hash.to_le_bytes());
    slot[8..12].copy_from_slice(&record.next_hash.to_le_bytes());
    slot[12..16].copy_from_slice(&record.route_hash.to_le_bytes());
    slot[16..18].copy_from_slice(&record.strength.to_le_bytes());
    slot[18..20].copy_from_slice(&record.accepted.to_le_bytes());
    slot[20..22].copy_from_slice(&record.rejected.to_le_bytes());
    slot[22..24].copy_from_slice(&record.flags.to_le_bytes());
    slot[24] = record.phase as u8;
    slot[25] = record.lens;
}

fn decode_record(slot: &[u8]) -> LlmWaveRecord {
    LlmWaveRecord {
        prefix_hash: u32::from_le_bytes(slot[0..4].try_into().unwrap_or_default()),
        token_hash: u32::from_le_bytes(slot[4..8].try_into().unwrap_or_default()),
        next_hash: u32::from_le_bytes(slot[8..12].try_into().unwrap_or_default()),
        route_hash: u32::from_le_bytes(slot[12..16].try_into().unwrap_or_default()),
        strength: i16::from_le_bytes(slot[16..18].try_into().unwrap_or_default()),
        accepted: u16::from_le_bytes(slot[18..20].try_into().unwrap_or_default()),
        rejected: u16::from_le_bytes(slot[20..22].try_into().unwrap_or_default()),
        flags: u16::from_le_bytes(slot[22..24].try_into().unwrap_or_default()),
        phase: slot[24] as i8,
        lens: slot[25],
    }
}

fn encode_vocabulary(vocabulary: &BTreeMap<u32, String>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(vocabulary.len() as u32).to_le_bytes());
    for (hash, token) in vocabulary {
        let token = token.as_bytes();
        if token.len() > u16::MAX as usize {
            continue;
        }
        bytes.extend_from_slice(&hash.to_le_bytes());
        bytes.extend_from_slice(&(token.len() as u16).to_le_bytes());
        bytes.extend_from_slice(token);
    }
    bytes
}

fn decode_vocabulary(bytes: &[u8]) -> io::Result<BTreeMap<u32, String>> {
    if bytes.len() < 4 {
        return Ok(BTreeMap::new());
    }
    let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap_or_default()) as usize;
    let mut cursor = 4;
    let mut vocabulary = BTreeMap::new();
    for _ in 0..count {
        if cursor + 6 > bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "truncated LLMWave vocabulary",
            ));
        }
        let hash = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or_default());
        let len = u16::from_le_bytes(bytes[cursor + 4..cursor + 6].try_into().unwrap_or_default())
            as usize;
        cursor += 6;
        if cursor + len > bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "truncated LLMWave vocabulary token",
            ));
        }
        let token = String::from_utf8(bytes[cursor..cursor + len].to_vec()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid LLMWave vocabulary utf8",
            )
        })?;
        cursor += len;
        vocabulary.insert(hash, token);
    }
    Ok(vocabulary)
}

fn prefix_hash(tokens: &[String]) -> u32 {
    let start = tokens.len().saturating_sub(3);
    hash32(&tokens[start..].join("\u{1f}"))
}

fn token_hash(token: &str) -> u32 {
    hash32(token)
}

fn route_hash_for(tokens: &[String]) -> u32 {
    hash32(&tokens.join("\u{1e}"))
}

fn phase_for(prefix: u32, token: u32, next: u32) -> i8 {
    ((prefix ^ token.rotate_left(7) ^ next.rotate_left(13)) as u8).wrapping_sub(128) as i8
}

fn hash32(text: &str) -> u32 {
    text.as_bytes().iter().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    })
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::super::signal::WordCandidate;
    use super::*;

    #[test]
    fn memory_packet_roundtrip_uses_32_byte_records() {
        let memory = LlmWaveMemory::from_text("html вот api html вот json");
        let bytes = encode_memory(&memory);
        assert!(bytes.len() > HEADER_BYTES + memory.records().len() * LLMWAVE_RECORD_BYTES);
        let decoded = decode_memory(&bytes).unwrap();
        assert_eq!(decoded.records(), memory.records());
        assert_eq!(decoded.vocabulary_len(), memory.vocabulary_len());
    }

    #[test]
    fn retrieves_next_token_from_phrase_memory() {
        let memory = LlmWaveMemory::from_text("html вот api");
        let prefix = tokenize("html");
        let (score, support) = memory.score_next_token(&prefix, "вот");
        assert!(score > 0.0);
        assert_eq!(support, 1);
    }

    #[test]
    fn shadow_scores_existing_l2_candidate_without_deciding_output() {
        let memory = LlmWaveMemory::from_text("html вот api");
        let candidates = vec![WordCandidate {
            text: "html вот api".to_string(),
            source: "LayoutWordCell32",
            energy: 0.7,
            risk: 0.1,
            support: vec![],
        }];
        let report = score_candidates("html djn api", &candidates, &memory);
        assert_eq!(report.top.as_ref().unwrap().source, "LayoutWordCell32");
    }

    #[test]
    fn predicts_phrase_continuation_from_l3_memory() {
        let memory = LlmWaveMemory::from_text(
            "на улице опять идёт дождь на улице опять идёт снег я хочу проверить автозамену",
        );
        let predictions = memory.predict_phrase("на улице опять идёт", 2, 4);
        assert!(predictions.iter().any(|item| item.text.contains("дожд")));
        assert!(predictions.iter().any(|item| item.tokens.len() >= 5));
    }

    #[test]
    fn longer_phrase_context_suppresses_short_wrong_association() {
        let memory = LlmWaveMemory::from_text(
            "идёт дом\nна улице опять идёт дождь\nна улице опять идёт снег",
        );
        let predictions = memory.predict_phrase("на улице опять идёт", 1, 4);
        assert!(predictions.iter().any(|item| item.text.ends_with("дождь")));
        assert!(predictions.iter().any(|item| item.text.ends_with("снег")));
        assert!(!predictions.iter().any(|item| item.text.ends_with("дом")));
    }

    #[test]
    fn next_token_score_backs_off_to_recent_phrase_modes() {
        let memory = LlmWaveMemory::from_text(
            "идёт дом\nна улице опять идёт дождь\nсегодня на улице опять идёт дождь",
        );
        let prefix = tokenize("на улице опять идёт");
        let rain = memory
            .score_next_token_report(&prefix, "дождь")
            .expect("rain should be supported by recent phrase modes");
        let house = memory
            .score_next_token_report(&prefix, "дом")
            .expect("short one-token association still exists");

        assert_eq!(rain.width, 3);
        assert_eq!(house.width, 1);
        assert!(
            rain.score > house.score,
            "longer phrase context must beat short association: rain={rain:?} house={house:?}"
        );
    }

    #[test]
    fn l3_candidate_scoring_uses_long_phrase_context() {
        let memory = LlmWaveMemory::from_text(
            "идёт дом\nна улице опять идёт дождь\nсегодня на улице опять идёт дождь",
        );
        let candidates = vec![
            WordCandidate {
                text: "на улице опять идёт дом".to_string(),
                source: "PhraseForecastCell32",
                energy: 0.7,
                risk: 0.1,
                support: vec![],
            },
            WordCandidate {
                text: "на улице опять идёт дождь".to_string(),
                source: "PhraseForecastCell32",
                energy: 0.7,
                risk: 0.1,
                support: vec![],
            },
        ];
        let report = score_candidates("на улице опять идёт д", &candidates, &memory);

        assert_eq!(
            report.top.as_ref().map(|item| item.text.as_str()),
            Some("на улице опять идёт дождь")
        );
    }

    #[test]
    fn report_contains_phrase_predictions_even_without_l2_candidates() {
        let memory = LlmWaveMemory::from_text("я хочу проверить автозамену");
        let report = score_candidates("я хочу", &[], &memory);
        assert!(report
            .predictions
            .iter()
            .any(|item| item.text.contains("проверить")));
    }

    #[test]
    fn learning_deltas_show_live_phrase_becoming_l3_route() {
        let seed = LlmWaveMemory::from_text("на улице опять идёт дождь");
        let combined = LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nя хочу проверить автозамену\nя хочу проверить режим",
        );
        let deltas = learning_deltas(
            &seed,
            &combined,
            ["я хочу проверить автозамену", "я хочу проверить режим"].into_iter(),
            4,
        );

        assert!(
            deltas.iter().any(|delta| {
                delta.prefix == "я хочу"
                    && delta.next_token == "проверить"
                    && delta.live_score > delta.seed_score
            }),
            "expected live phrase to create an L3 prefix->next delta, got {deltas:?}"
        );
    }

    #[test]
    fn phrase_forecast_completes_partial_next_word() {
        let memory = LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nна улице опять идёт снег\nя хочу проверить автозамену",
        );
        let candidates = phrase_forecast_candidates("на улице опять идёт д", &memory);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == super::super::context_wave::PHRASE_FORECAST_CELL
                && candidate.text == "на улице опять идёт дождь"
        }));
        assert!(!candidates
            .iter()
            .any(|candidate| candidate.text == "на улице опять идёт дом"));
    }

    #[test]
    fn phrase_experience_keeps_only_phrase_boundaries() {
        assert!(phrase_experience("space", "я хочу проверить режим").is_some());
        assert!(phrase_experience("key", "я хочу проверить режим").is_none());
        assert!(phrase_experience("space", "git checkout -b test").is_none());
        assert!(phrase_experience("space", "https://example.test token").is_none());
        assert!(phrase_experience("space", "a = b").is_none());
    }

    #[test]
    fn phrase_experience_jsonl_loads_training_text() {
        let tmp =
            std::env::temp_dir().join(format!("lay-llmwave-exp-{}.jsonl", std::process::id()));
        let first = phrase_experience("space", "я хочу проверить режим").unwrap();
        let second = phrase_experience("space", "на улице опять идёт дождь").unwrap();
        let text = format!(
            "{}\n{}\nnot-json\n",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        crate::private_file::write_private_text(&tmp, &text).unwrap();

        let loaded = load_phrase_experience_text(&tmp).unwrap();
        let _ = std::fs::remove_file(tmp);

        assert!(loaded.contains("я хочу проверить режим"));
        assert!(loaded.contains("на улице опять идёт дождь"));
        assert!(!loaded.contains("not-json"));
    }

    #[test]
    fn phrase_experience_rejects_dirty_live_layout_fragments() {
        assert_eq!(
            phrase_experience_rejection_reason("space", "cjnhblybwf прислала скриншот"),
            Some(PhraseExperienceRejectReason::DirtyLayout)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "ghbdtn lfdfq тут"),
            Some(PhraseExperienceRejectReason::DirtyLayout)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", ", он уже там"),
            Some(PhraseExperienceRejectReason::UnsafeText)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "ОПА А тут"),
            Some(PhraseExperienceRejectReason::UnstableOrNoisy)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "Вообще делаей проект рефакторинга"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "Да короче он аможет любые"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "читай что в"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "Давай ей написаем чт"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "Давай ей написем ответ"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
    }

    #[test]
    fn phrase_experience_rejects_short_live_typo_tokens() {
        assert_eq!(
            phrase_experience_rejection_reason(
                "space",
                "хорошо ты обучил модель отличные подсказки ты гед"
            ),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "просо из за того что"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
        assert_eq!(
            phrase_experience_rejection_reason("space", "6 страниц нужно како-то софт"),
            Some(PhraseExperienceRejectReason::LowLanguageQuality)
        );
    }

    #[test]
    fn phrase_experience_keeps_mixed_technical_phrases() {
        assert!(phrase_experience("space", "html api работает").is_some());
        assert!(phrase_experience("space", "file тоже хорошо").is_some());
        assert!(phrase_experience("space", "Вообще делай проект рефакторинга").is_some());
    }
}
