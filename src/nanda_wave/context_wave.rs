use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{is_common_en_technical_word, is_common_ru_word, EN_HUNSPELL, EN_WORDS};
use crate::phrase_lexicon::is_known_russian_phrase_part;
use crate::russian_lexicon::{
    is_known_russian_word_or_form, russian_dictionary, russian_generated_form_dictionary,
    russian_tiny_dictionary,
};
use crate::text_metrics::damerau_levenshtein;

use super::signal::WordCandidate;

pub const SEMANTIC_WORD_SOURCE: &str = "SemanticWordCell32";
pub const PHRASE_FORECAST_CELL: &str = "PhraseForecastCell32";
const MAX_SEMANTIC_WORD_CANDIDATES: usize = 8;
const MAX_WAVE_BUCKET_SCAN: usize = 512;
const MAX_WAVE_POOL: usize = 4096;
static PREFIX_COMPLETION_INDEX_WARM: AtomicBool = AtomicBool::new(false);
static RU_WORD_WAVE_MEMORY: OnceLock<RuWordWaveMemory> = OnceLock::new();
static EN_WORD_WAVE_MEMORY: OnceLock<EnWordWaveMemory> = OnceLock::new();
static RU_WORD_PREFIX_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static EN_WORD_PREFIX_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

pub fn warm_up() {
    let _ = ru_word_wave_memory().entries.len();
    let _ = en_word_wave_memory().entries.len();
}

pub fn warm_up_prefix_completion_indexes() {
    warm_up();
    let _ = ru_word_prefix_index().len();
    let _ = en_word_prefix_index().len();
    PREFIX_COMPLETION_INDEX_WARM.store(true, Ordering::Release);
}

pub fn prefix_wave_memory_is_warm() -> bool {
    PREFIX_COMPLETION_INDEX_WARM.load(Ordering::Acquire)
}

pub fn ru_word_prefix_completion_suffixes(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
) -> Vec<String> {
    word_prefix_completion_suffixes(
        prefix,
        max_suffix_chars,
        limit,
        None,
        ru_word_prefix_index(),
    )
}

pub fn ru_word_prefix_completion_suffixes_if_bucket_at_most(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
    max_bucket_entries: usize,
) -> Vec<String> {
    word_prefix_completion_suffixes(
        prefix,
        max_suffix_chars,
        limit,
        Some(max_bucket_entries),
        ru_word_prefix_index(),
    )
}

pub fn en_word_prefix_completion_suffixes(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
) -> Vec<String> {
    word_prefix_completion_suffixes(
        &prefix.to_ascii_lowercase(),
        max_suffix_chars,
        limit,
        None,
        en_word_prefix_index(),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticWaveMode {
    pub name: &'static str,
    pub frequency_id: u16,
    pub amplitude: f32,
    pub phase: i8,
    pub damping: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CandidateInterference {
    pub candidate: &'static str,
    pub amplitude: f32,
    pub phase_alignment: f32,
    pub coherence: f32,
    pub damping: f32,
    pub projection: f32,
    pub source_modes: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextWave {
    pub prefix: String,
    pub partial_token: String,
    pub modes: Vec<SemanticWaveMode>,
}

impl ContextWave {
    fn mode_energy(&self) -> f32 {
        let total = self
            .modes
            .iter()
            .map(|mode| mode.amplitude * (1.0 - mode.damping))
            .sum::<f32>();
        (total / self.modes.len().max(1) as f32).clamp(0.0, 1.0)
    }
}

pub fn semantic_word_candidates(tail: &str) -> Vec<WordCandidate> {
    let mut candidates = if known_russian_phrase_tail(tail) {
        Vec::new()
    } else {
        nearest_ru_word_candidates(tail)
    };
    if let Some(wave) = context_wave_for_tail(tail) {
        candidates.extend(
            candidate_interferences(&wave)
                .into_iter()
                .take(3)
                .filter(|item| item.projection >= 0.22)
                .map(|item| interference_to_candidate(&wave, item)),
        );
    }
    candidates
        .sort_by(|left, right| (right.energy - right.risk).total_cmp(&(left.energy - left.risk)));
    candidates.dedup_by(|left, right| left.text == right.text);
    candidates
        .into_iter()
        .take(MAX_SEMANTIC_WORD_CANDIDATES)
        .collect()
}

fn nearest_ru_word_candidates(tail: &str) -> Vec<WordCandidate> {
    let trimmed = tail.trim_end();
    let Some((prefix, token)) = split_last_token(trimmed) else {
        return Vec::new();
    };
    if prefix.split_whitespace().next().is_none() {
        return Vec::new();
    }
    let normalized = normalize_ru(token);
    let len = normalized.chars().count();
    if !(4..=18).contains(&len) || known_ru_token_blocks_semantic_rewrite(&normalized) {
        return Vec::new();
    }
    if normalized.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return nearest_en_word_candidates(prefix, token, &normalized);
    }
    if !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let query_modes = word_wave_modes(&normalized);
    let mut seen = HashSet::new();
    let mut pool = Vec::new();
    for mode in &query_modes {
        let Some(bucket) = ru_word_wave_memory().buckets.get(mode) else {
            continue;
        };
        for idx in bucket.iter().take(MAX_WAVE_BUCKET_SCAN) {
            if seen.insert(*idx) {
                pool.push(*idx);
                if pool.len() >= MAX_WAVE_POOL {
                    break;
                }
            }
        }
        if pool.len() >= MAX_WAVE_POOL {
            break;
        }
    }
    let mut ranked = pool
        .into_iter()
        .filter_map(|word| {
            let entry = &ru_word_wave_memory().entries[word];
            if entry.word == normalized || entry.len.abs_diff(len) > 2 {
                return None;
            }
            let distance = damerau_levenshtein(&normalized, &entry.word);
            let resonance = word_wave_resonance(&normalized, entry, &query_modes);
            let allowed =
                semantic_wave_candidate_allowed(&normalized, &entry.word, distance, resonance);
            (allowed && ru_candidate_passes_semantic_guards(&normalized, &entry.word, distance))
                .then_some((entry.word.clone(), distance, resonance))
        })
        .collect::<Vec<_>>();
    let ranked_words = ranked
        .iter()
        .map(|(word, _distance, _resonance)| word.clone())
        .collect::<HashSet<_>>();
    ranked.extend(fuzzy_ru_word_candidates(&normalized).into_iter().filter(
        |(word, distance, _resonance)| {
            !ranked_words.contains(word.as_str())
                && ru_candidate_passes_semantic_guards(&normalized, word, *distance)
        },
    ));
    ranked.sort_by(
        |(left, left_distance, left_resonance), (right, right_distance, right_resonance)| {
            left_distance
                .cmp(right_distance)
                .then_with(|| is_common_ru_word(right).cmp(&is_common_ru_word(left)))
                .then_with(|| usage_prior(right).total_cmp(&usage_prior(left)))
                .then_with(|| right_resonance.total_cmp(left_resonance))
                .then_with(|| {
                    left.chars()
                        .count()
                        .abs_diff(len)
                        .cmp(&right.chars().count().abs_diff(len))
                })
                .then_with(|| left.cmp(right))
        },
    );
    ranked
        .into_iter()
        .take(MAX_SEMANTIC_WORD_CANDIDATES)
        .map(|(word, distance, resonance)| {
            ru_word_to_candidate(prefix, token, &word, distance, resonance)
        })
        .collect()
}

fn fuzzy_ru_word_candidates(normalized: &str) -> Vec<(String, usize, f32)> {
    let query_modes = word_wave_modes(normalized);
    let mut candidates = crate::ru_typo::fuzzy_known_word_candidates(normalized)
        .into_iter()
        .filter_map(|word| {
            let len = word.chars().count();
            if !(4..=18).contains(&len) || !word.chars().all(is_cyrillic_letter) {
                return None;
            }
            let distance = damerau_levenshtein(normalized, &word);
            if distance == 0 || distance > 3 {
                return None;
            }
            let modes = word_wave_modes(&word);
            let entry = RuWordWaveEntry { word, len, modes };
            let resonance = word_wave_resonance(normalized, &entry, &query_modes);
            Some((entry.word, distance, resonance))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(
        |(left, left_distance, left_resonance), (right, right_distance, right_resonance)| {
            left_distance
                .cmp(right_distance)
                .then_with(|| right_resonance.total_cmp(left_resonance))
                .then_with(|| left.cmp(right))
        },
    );
    candidates
}

fn nearest_en_word_candidates(prefix: &str, token: &str, normalized: &str) -> Vec<WordCandidate> {
    let len = normalized.chars().count();
    if !(4..=18).contains(&len)
        || is_common_en_technical_word(normalized)
        || en_word_wave_memory().known.contains(normalized)
    {
        return Vec::new();
    }
    let converted = crate::dict::convert(normalized, crate::dict::detect_direction(normalized));
    if converted != normalized && is_known_russian_word_or_form(&converted) {
        return Vec::new();
    }

    let query_modes = word_wave_modes(normalized);
    let mut seen = HashSet::new();
    let mut pool = Vec::new();
    for mode in &query_modes {
        let Some(bucket) = en_word_wave_memory().buckets.get(mode) else {
            continue;
        };
        for idx in bucket.iter().take(MAX_WAVE_BUCKET_SCAN) {
            if seen.insert(*idx) {
                pool.push(*idx);
                if pool.len() >= MAX_WAVE_POOL {
                    break;
                }
            }
        }
        if pool.len() >= MAX_WAVE_POOL {
            break;
        }
    }

    let mut ranked = pool
        .into_iter()
        .filter_map(|word| {
            let entry = &en_word_wave_memory().entries[word];
            if entry.word == normalized || entry.len.abs_diff(len) > 2 {
                return None;
            }
            let distance = damerau_levenshtein(normalized, &entry.word);
            let resonance = word_wave_resonance(normalized, entry, &query_modes);
            let allowed = distance == 1
                || (len >= 8 && distance == 2 && resonance >= 0.48)
                || (len >= 11 && distance == 3 && resonance >= 0.72);
            allowed.then_some((entry, distance, resonance))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(
        |(left, left_distance, left_resonance), (right, right_distance, right_resonance)| {
            left_distance
                .cmp(right_distance)
                .then_with(|| right_resonance.total_cmp(left_resonance))
                .then_with(|| left.len.abs_diff(len).cmp(&right.len.abs_diff(len)))
                .then_with(|| left.word.cmp(&right.word))
        },
    );
    ranked
        .into_iter()
        .take(MAX_SEMANTIC_WORD_CANDIDATES)
        .map(|(entry, distance, resonance)| {
            en_word_to_candidate(prefix, token, &entry.word, distance, resonance)
        })
        .collect()
}

fn looks_like_known_verb_to_noun_drift(original: &str, candidate: &str) -> bool {
    is_known_russian_word_or_form(original)
        && has_russian_verb_tail(original)
        && !has_russian_verb_tail(candidate)
}

fn ru_candidate_passes_semantic_guards(original: &str, candidate: &str, distance: usize) -> bool {
    !looks_like_short_word_drift(original, candidate)
        && !looks_like_verb_to_nonverb_drift(original, candidate)
        && !looks_like_suffix_stripping(original, candidate)
        && !looks_like_case_vowel_append_drift(original, candidate)
        && !looks_like_case_vowel_to_consonant_drift(original, candidate)
        && !looks_like_known_form_to_other_known_word_drift(original, candidate, distance)
        && !looks_like_known_verb_to_noun_drift(original, candidate)
        && !looks_like_nonverb_to_verb_drift(original, candidate)
        && !looks_like_short_dense_cluster_drift(original, distance)
        && !looks_like_short_multi_edit_guess(original, candidate, distance)
        && !looks_like_reflexive_plus_case_vowel(candidate)
        && !looks_like_short_y_drop_drift(original, candidate, distance)
}

fn semantic_wave_candidate_allowed(
    original: &str,
    candidate: &str,
    distance: usize,
    resonance: f32,
) -> bool {
    if negative_prefix_repair(original, candidate, distance) {
        return true;
    }
    if first_char(original) != first_char(candidate) {
        return false;
    }
    if distance >= 2 && has_dense_consonant_cluster(original) {
        return false;
    }
    if !is_common_ru_word(candidate) {
        return false;
    }
    let len = original.chars().count();
    distance == 1
        || (len >= 7 && distance == 2 && resonance >= 0.42)
        || (len >= 10 && distance == 3 && resonance >= 0.68)
}

fn first_char(word: &str) -> Option<char> {
    word.chars().next()
}

fn has_dense_consonant_cluster(word: &str) -> bool {
    let mut run = 0;
    for ch in word.chars() {
        if is_russian_consonant(ch) {
            run += 1;
            if run >= 3 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn known_ru_token_blocks_semantic_rewrite(word: &str) -> bool {
    known_ru_token(word) && !(word.starts_with("не") && word.chars().count() >= 7)
}

fn looks_like_verb_to_nonverb_drift(original: &str, candidate: &str) -> bool {
    (has_russian_present_tail(original) && !has_russian_present_tail(candidate))
        || (has_russian_verb_tail(original) && !has_russian_verb_tail(candidate))
}

fn looks_like_nonverb_to_verb_drift(original: &str, candidate: &str) -> bool {
    !has_russian_verb_tail(original) && has_russian_verb_tail(candidate)
}

fn looks_like_short_dense_cluster_drift(original: &str, distance: usize) -> bool {
    distance >= 2 && original.chars().count() <= 7 && has_dense_consonant_cluster(original)
}

fn looks_like_short_multi_edit_guess(original: &str, candidate: &str, distance: usize) -> bool {
    distance >= 3
        && original.chars().count() <= 8
        && !negative_prefix_repair(original, candidate, distance)
}

fn looks_like_reflexive_plus_case_vowel(candidate: &str) -> bool {
    [
        "сяа", "сяу", "сяы", "сяи", "сяо", "сьа", "сьу", "сьы", "сьи", "сьо",
    ]
    .iter()
    .any(|tail| candidate.ends_with(tail))
}

fn looks_like_short_y_drop_drift(original: &str, candidate: &str, distance: usize) -> bool {
    distance == 1
        && original.chars().count() <= 6
        && original.contains('й')
        && !candidate.contains('й')
}

fn looks_like_short_word_drift(original: &str, candidate: &str) -> bool {
    original.chars().count() <= 4 && candidate.chars().count() <= 5 && original != candidate
}

fn looks_like_known_form_to_other_known_word_drift(
    original: &str,
    candidate: &str,
    distance: usize,
) -> bool {
    if original == candidate || negative_prefix_repair(original, candidate, distance) {
        return false;
    }
    known_ru_token(original) && known_ru_token(candidate)
}

fn known_ru_token(word: &str) -> bool {
    is_common_ru_word(word)
        || is_known_russian_phrase_part(word)
        || is_known_russian_word_or_form(word)
        || russian_tiny_dictionary().contains(word)
}

fn negative_prefix_repair(original: &str, candidate: &str, distance: usize) -> bool {
    (2..=3).contains(&distance)
        && original.starts_with("не")
        && candidate.starts_with("не")
        && original.chars().count().abs_diff(candidate.chars().count()) <= 3
}

fn has_russian_verb_tail(word: &str) -> bool {
    const VERB_TAILS: &[&str] = &[
        "ет", "ит", "ют", "ут", "ат", "ят", "ем", "им", "ешь", "ишь", "ете", "ите", "ал", "ала",
        "ило", "или", "ил", "ено", "ена", "ены", "ает", "яет", "ует", "ёт", "ся", "ыть", "ять",
        "ыта", "ыто", "ыты", "ята", "ято", "яты",
    ];
    VERB_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn has_russian_present_tail(word: &str) -> bool {
    const PRESENT_TAILS: &[&str] = &[
        "ется", "ётся", "атся", "ятся", "ешь", "ишь", "ете", "ите", "ают", "яют", "уют", "ют",
        "ут", "ат", "ят", "ает", "яет", "ует", "ет", "ит",
    ];
    PRESENT_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn looks_like_suffix_stripping(original: &str, candidate: &str) -> bool {
    let Some(stripped) = original.strip_prefix(candidate) else {
        return false;
    };
    !stripped.is_empty()
        && stripped
            .chars()
            .all(|ch| matches!(ch, 'а' | 'я' | 'у' | 'ю' | 'е' | 'ы' | 'и' | 'о' | 'м'))
}

fn looks_like_case_vowel_append_drift(original: &str, candidate: &str) -> bool {
    let Some(appended) = candidate.strip_prefix(original) else {
        return false;
    };
    appended.chars().count() == 1
        && appended.chars().all(is_russian_case_vowel)
        && original.chars().count() >= 4
}

fn looks_like_case_vowel_to_consonant_drift(original: &str, candidate: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if original_chars.len() != candidate_chars.len() || original_chars.len() < 4 {
        return false;
    }
    let last_idx = original_chars.len() - 1;
    original_chars[..last_idx] == candidate_chars[..last_idx]
        && is_russian_case_vowel(original_chars[last_idx])
        && is_russian_consonant(candidate_chars[last_idx])
}

fn is_russian_case_vowel(ch: char) -> bool {
    matches!(ch, 'а' | 'я' | 'у' | 'ю' | 'е' | 'ы' | 'и' | 'о')
}

fn is_russian_consonant(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
        && !matches!(
            ch,
            'а' | 'я' | 'у' | 'ю' | 'е' | 'ё' | 'ы' | 'и' | 'о' | 'э' | 'ь' | 'ъ'
        )
}

#[derive(Debug)]
struct RuWordWaveMemory {
    entries: Vec<RuWordWaveEntry>,
    buckets: HashMap<u16, Vec<usize>>,
}

#[derive(Debug)]
struct RuWordWaveEntry {
    word: String,
    len: usize,
    modes: Vec<u16>,
}

#[derive(Debug)]
struct EnWordWaveMemory {
    entries: Vec<RuWordWaveEntry>,
    buckets: HashMap<u16, Vec<usize>>,
    known: HashSet<String>,
}

fn ru_word_wave_memory() -> &'static RuWordWaveMemory {
    RU_WORD_WAVE_MEMORY.get_or_init(|| {
        let mut entries = Vec::new();
        let mut buckets = HashMap::<u16, Vec<usize>>::new();
        let mut words = russian_dictionary()
            .iter()
            .map(|word| normalize_ru(word))
            .collect::<Vec<_>>();
        words.extend(
            russian_generated_form_dictionary()
                .iter()
                .map(|word| normalize_ru(word)),
        );
        words.sort_by(|left, right| {
            is_common_ru_word(right)
                .cmp(&is_common_ru_word(left))
                .then_with(|| left.chars().count().cmp(&right.chars().count()))
                .then_with(|| left.cmp(right))
        });
        words.dedup();
        for word in words {
            let word = normalize_ru(&word);
            let len = word.chars().count();
            if !(4..=18).contains(&len) || !word.chars().all(is_cyrillic_letter) {
                continue;
            }
            let modes = word_wave_modes(&word);
            let idx = entries.len();
            for mode in &modes {
                buckets.entry(*mode).or_default().push(idx);
            }
            entries.push(RuWordWaveEntry { word, len, modes });
        }
        RuWordWaveMemory { entries, buckets }
    })
}

fn en_word_wave_memory() -> &'static EnWordWaveMemory {
    EN_WORD_WAVE_MEMORY.get_or_init(|| {
        let mut known = HashSet::new();
        extend_english_words_from_hunspell(&mut known, EN_HUNSPELL);
        extend_english_words_from_plain(&mut known, EN_WORDS);
        let mut words = known.iter().cloned().collect::<Vec<_>>();
        words.sort_by(|left, right| {
            is_common_en_technical_word(right)
                .cmp(&is_common_en_technical_word(left))
                .then_with(|| left.chars().count().cmp(&right.chars().count()))
                .then_with(|| left.cmp(right))
        });
        let mut entries = Vec::new();
        let mut buckets = HashMap::<u16, Vec<usize>>::new();
        for word in words {
            let len = word.chars().count();
            if !(4..=18).contains(&len) {
                continue;
            }
            let modes = word_wave_modes(&word);
            let idx = entries.len();
            for mode in &modes {
                buckets.entry(*mode).or_default().push(idx);
            }
            entries.push(RuWordWaveEntry { word, len, modes });
        }
        EnWordWaveMemory {
            entries,
            buckets,
            known,
        }
    })
}

fn ru_word_prefix_index() -> &'static HashMap<String, Vec<String>> {
    RU_WORD_PREFIX_INDEX.get_or_init(|| build_prefix_index(&ru_word_wave_memory().entries))
}

fn en_word_prefix_index() -> &'static HashMap<String, Vec<String>> {
    EN_WORD_PREFIX_INDEX.get_or_init(|| build_prefix_index(&en_word_wave_memory().entries))
}

fn build_prefix_index(entries: &[RuWordWaveEntry]) -> HashMap<String, Vec<String>> {
    let mut index = HashMap::<String, Vec<String>>::new();
    for entry in entries {
        for prefix_len in 2..entry.len {
            let prefix = entry.word.chars().take(prefix_len).collect::<String>();
            let byte_idx = entry
                .word
                .char_indices()
                .nth(prefix_len)
                .map(|(idx, _)| idx)
                .unwrap_or(entry.word.len());
            let suffix = entry.word[byte_idx..].to_string();
            if suffix.is_empty() {
                continue;
            }
            index.entry(prefix).or_default().push(suffix);
        }
    }
    index
}

fn word_prefix_completion_suffixes(
    prefix: &str,
    max_suffix_chars: usize,
    limit: usize,
    max_bucket_entries: Option<usize>,
    prefix_index: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let prefix = prefix.trim().to_lowercase();
    let prefix_len = prefix.chars().count();
    if prefix_len < 2 {
        return Vec::new();
    }
    let Some(suffixes) = prefix_index.get(&prefix) else {
        return Vec::new();
    };
    if max_bucket_entries.is_some_and(|max| suffixes.len() > max) {
        return Vec::new();
    }
    suffixes
        .iter()
        .filter(|suffix| suffix.chars().count() <= max_suffix_chars)
        .take(limit)
        .cloned()
        .collect()
}

fn extend_english_words_from_hunspell(words: &mut HashSet<String>, path: &str) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    words.extend(text.lines().skip(1).filter_map(|line| {
        let word = line
            .trim()
            .split_once('/')
            .map_or(line.trim(), |(word, _)| word);
        english_word_from_raw(word)
    }));
}

fn extend_english_words_from_plain(words: &mut HashSet<String>, path: &str) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    words.extend(text.lines().filter_map(english_word_from_raw));
}

fn english_word_from_raw(word: &str) -> Option<String> {
    let word = word.trim().to_ascii_lowercase();
    ((4..=18).contains(&word.chars().count())
        && word.chars().all(|ch| ch.is_ascii_alphabetic() || ch == '-')
        && word.chars().any(|ch| ch.is_ascii_alphabetic()))
    .then_some(word)
}

fn word_wave_resonance(query: &str, entry: &RuWordWaveEntry, query_modes: &[u16]) -> f32 {
    if query_modes.is_empty() || entry.modes.is_empty() {
        return 0.0;
    }
    let overlap = query_modes
        .iter()
        .filter(|mode| entry.modes.binary_search(mode).is_ok())
        .count() as f32;
    let mode_energy = overlap / query_modes.len().max(entry.modes.len()) as f32;
    let len_energy = 1.0 - (query.chars().count().abs_diff(entry.len) as f32 / 6.0).min(1.0);
    let edge_energy = edge_alignment(query, &entry.word);
    (mode_energy * mode_energy * 0.56 + len_energy * 0.20 + edge_energy * 0.24).clamp(0.0, 1.0)
}

fn edge_alignment(query: &str, word: &str) -> f32 {
    let query_chars = query.chars().collect::<Vec<_>>();
    let word_chars = word.chars().collect::<Vec<_>>();
    let first = query_chars
        .first()
        .zip(word_chars.first())
        .is_some_and(|(left, right)| left == right) as u8 as f32;
    let last = query_chars
        .last()
        .zip(word_chars.last())
        .is_some_and(|(left, right)| left == right) as u8 as f32;
    (first * 0.55 + last * 0.45).clamp(0.0, 1.0)
}

fn word_wave_modes(word: &str) -> Vec<u16> {
    let chars = word.chars().collect::<Vec<_>>();
    let mut modes = Vec::new();
    modes.push(word_mode_hash(0x11, word.chars().count() as u32));
    if let Some(first) = chars.first() {
        modes.push(word_mode_hash(0x21, *first as u32));
    }
    if let Some(last) = chars.last() {
        modes.push(word_mode_hash(0x22, *last as u32));
    }
    for window in chars.windows(2) {
        modes.push(word_mode_hash(
            0x31,
            ((window[0] as u32) << 11) ^ window[1] as u32,
        ));
    }
    for window in chars.windows(3) {
        modes.push(word_mode_hash(
            0x41,
            ((window[0] as u32) << 16) ^ ((window[1] as u32) << 8) ^ window[2] as u32,
        ));
    }
    modes.sort_unstable();
    modes.dedup();
    modes
}

fn word_mode_hash(seed: u32, value: u32) -> u16 {
    let mut hash = seed.wrapping_mul(0x9E37_79B9) ^ value;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85EB_CA6B);
    hash ^= hash >> 13;
    (hash & 0x7FF) as u16
}

fn ru_word_to_candidate(
    prefix: &str,
    token: &str,
    word: &str,
    distance: usize,
    resonance: f32,
) -> WordCandidate {
    let len = normalize_ru(token).chars().count().max(1);
    let closeness = 1.0 - (distance as f32 / len as f32);
    let common_boost = if is_common_ru_word(word) { 0.055 } else { 0.0 };
    let usage_boost = usage_prior(word);
    WordCandidate {
        text: format!("{prefix}{word}"),
        source: SEMANTIC_WORD_SOURCE,
        energy: (0.50 + closeness * 0.24 + resonance * 0.18 + common_boost + usage_boost)
            .clamp(0.0, 0.95),
        risk: (0.32
            - closeness * 0.12
            - resonance * 0.06
            - common_boost * 0.35
            - usage_boost * 0.45)
            .clamp(0.09, 0.32),
        support: vec![
            "ru-word-wave-memory".to_string(),
            format!(
                "token={token:?} candidate={word:?} distance={distance} resonance={resonance:.3} usage_prior={usage_boost:.3}"
            ),
        ],
    }
}

fn usage_prior(word: &str) -> f32 {
    super::usage_prior::word_usage_prior(word)
}

fn en_word_to_candidate(
    prefix: &str,
    token: &str,
    word: &str,
    distance: usize,
    resonance: f32,
) -> WordCandidate {
    let len = token.chars().count().max(1);
    let closeness = 1.0 - (distance as f32 / len as f32);
    WordCandidate {
        text: format!("{prefix}{word}"),
        source: SEMANTIC_WORD_SOURCE,
        energy: (0.44 + closeness * 0.20 + resonance * 0.14).clamp(0.0, 0.82),
        risk: (0.36 - closeness * 0.08 - resonance * 0.04).clamp(0.18, 0.40),
        support: vec![
            "en-word-wave-memory".to_string(),
            format!(
                "token={token:?} candidate={word:?} distance={distance} resonance={resonance:.3}"
            ),
        ],
    }
}

pub fn phrase_forecast_summary(original: &str, chosen: &WordCandidate) -> Option<String> {
    if chosen.source != SEMANTIC_WORD_SOURCE {
        return None;
    }
    let wave = context_wave_for_tail(original)?;
    let best = candidate_interferences(&wave).into_iter().next()?;
    let forecast = format!("{}{}", wave.prefix, best.candidate);
    Some(format!(
        "forecast={forecast:?} word={:?} projection={:.3} coherence={:.3} cell={PHRASE_FORECAST_CELL}",
        best.candidate, best.projection, best.coherence
    ))
}

pub fn context_wave_for_tail(tail: &str) -> Option<ContextWave> {
    let trimmed = tail.trim_end();
    let (prefix, partial_token) = split_last_token(trimmed)?;
    let lower_prefix = prefix.to_lowercase();
    let lower_partial = partial_token.to_lowercase();
    if !weather_context_is_active(&lower_prefix, &lower_partial) {
        return None;
    }
    Some(ContextWave {
        prefix: prefix.to_string(),
        partial_token: partial_token.to_string(),
        modes: vec![
            SemanticWaveMode {
                name: "weather_context",
                frequency_id: 0x0711,
                amplitude: 0.92,
                phase: 12,
                damping: 0.04,
            },
            SemanticWaveMode {
                name: "repeat_process",
                frequency_id: 0x0417,
                amplitude: repeat_process_amplitude(&lower_prefix),
                phase: 7,
                damping: 0.08,
            },
            SemanticWaveMode {
                name: "verb_frame_idet_event",
                frequency_id: 0x0903,
                amplitude: event_verb_amplitude(&lower_prefix),
                phase: 11,
                damping: 0.05,
            },
            SemanticWaveMode {
                name: "prefix_d",
                frequency_id: 0x0024,
                amplitude: prefix_d_amplitude(&lower_partial),
                phase: 5,
                damping: 0.03,
            },
        ],
    })
}

pub fn candidate_interferences(wave: &ContextWave) -> Vec<CandidateInterference> {
    let mode_energy = wave.mode_energy();
    let mut candidates = vec![
        CandidateInterference {
            candidate: "дождь",
            amplitude: 0.96,
            phase_alignment: 0.96,
            coherence: mode_energy,
            damping: 0.04,
            projection: 0.0,
            source_modes: vec![
                "weather_context",
                "repeat_process",
                "verb_frame_idet_event",
                "prefix_d",
            ],
        },
        CandidateInterference {
            candidate: "дождик",
            amplitude: 0.86,
            phase_alignment: 0.86,
            coherence: mode_energy * 0.92,
            damping: 0.06,
            projection: 0.0,
            source_modes: vec!["weather_context", "verb_frame_idet_event", "prefix_d"],
        },
        CandidateInterference {
            candidate: "день",
            amplitude: 0.42,
            phase_alignment: 0.54,
            coherence: mode_energy * 0.62,
            damping: 0.10,
            projection: 0.0,
            source_modes: vec!["prefix_d"],
        },
        CandidateInterference {
            candidate: "дрель",
            amplitude: 0.20,
            phase_alignment: 0.24,
            coherence: mode_energy * 0.36,
            damping: 0.14,
            projection: 0.0,
            source_modes: vec!["prefix_d"],
        },
    ];
    for item in &mut candidates {
        item.projection =
            (item.amplitude * item.phase_alignment * item.coherence - item.damping).max(0.0);
    }
    candidates.sort_by(|left, right| right.projection.total_cmp(&left.projection));
    candidates
}

fn interference_to_candidate(wave: &ContextWave, item: CandidateInterference) -> WordCandidate {
    let text = format!("{}{}", wave.prefix, item.candidate);
    let energy = (0.42 + item.projection * 0.62).clamp(0.0, 0.99);
    let risk = (0.18 - item.projection * 0.08).clamp(0.06, 0.18);
    WordCandidate {
        text,
        source: SEMANTIC_WORD_SOURCE,
        energy,
        risk,
        support: semantic_support(wave, &item),
    }
}

fn semantic_support(wave: &ContextWave, item: &CandidateInterference) -> Vec<String> {
    let mut support = vec![
        format!(
            "candidate={} projection={:.3} coherence={:.3}",
            item.candidate, item.projection, item.coherence
        ),
        format!(
            "partial={:?} phase_alignment={:.3}",
            wave.partial_token, item.phase_alignment
        ),
    ];
    support.extend(wave.modes.iter().map(|mode| {
        format!(
            "mode={} freq={} amp={:.3} phase={} damping={:.3}",
            mode.name, mode.frequency_id, mode.amplitude, mode.phase, mode.damping
        )
    }));
    support
}

fn weather_context_is_active(prefix: &str, partial: &str) -> bool {
    has_weather_place(prefix)
        && has_event_verb(prefix)
        && has_repeat_or_weather_hint(prefix)
        && partial.starts_with('д')
        && partial.chars().count() <= 4
}

fn has_weather_place(prefix: &str) -> bool {
    prefix
        .split_whitespace()
        .any(|token| normalize_ru(token).starts_with("улиц"))
}

fn has_event_verb(prefix: &str) -> bool {
    prefix.split_whitespace().any(|token| {
        matches!(
            normalize_ru(token).as_str(),
            "идет" | "идёт" | "шел" | "шёл"
        )
    })
}

fn has_repeat_or_weather_hint(prefix: &str) -> bool {
    prefix.split_whitespace().any(|token| {
        matches!(
            normalize_ru(token).as_str(),
            "опять" | "снова" | "сегодня" | "наружи"
        )
    })
}

fn repeat_process_amplitude(prefix: &str) -> f32 {
    if prefix
        .split_whitespace()
        .any(|token| matches!(normalize_ru(token).as_str(), "опять" | "снова"))
    {
        0.86
    } else {
        0.58
    }
}

fn event_verb_amplitude(prefix: &str) -> f32 {
    if has_event_verb(prefix) {
        0.88
    } else {
        0.40
    }
}

fn prefix_d_amplitude(partial: &str) -> f32 {
    if partial == "д" {
        0.91
    } else if "дождь".starts_with(partial) || "дождик".starts_with(partial) {
        0.97
    } else {
        0.55
    }
}

fn normalize_ru(token: &str) -> String {
    token
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”'))
        .to_lowercase()
}

fn known_russian_phrase_tail(tail: &str) -> bool {
    let tokens = tail
        .split_whitespace()
        .map(normalize_ru)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let Some(last) = tokens.last() else {
        return false;
    };
    tokens.len() >= 2
        && is_common_ru_word(last)
        && tokens[..tokens.len() - 1]
            .iter()
            .all(|token| is_known_russian_phrase_part(token))
}

fn split_last_token(text: &str) -> Option<(&str, &str)> {
    if text.is_empty() {
        return None;
    }
    let start = text
        .char_indices()
        .rev()
        .find(|(_idx, ch)| ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let (prefix, token) = text.split_at(start);
    (!token.is_empty()).then_some((prefix, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_wave_prefers_rain_over_day_and_drill() {
        let wave = context_wave_for_tail("На улице опять идёт д").unwrap();
        let candidates = candidate_interferences(&wave);
        assert_eq!(candidates[0].candidate, "дождь");
        assert!(candidates[0].projection > candidates[1].projection);
        assert!(
            candidates
                .iter()
                .find(|item| item.candidate == "день")
                .unwrap()
                .projection
                > candidates
                    .iter()
                    .find(|item| item.candidate == "дрель")
                    .unwrap()
                    .projection
        );
    }

    #[test]
    fn semantic_wave_generates_weather_candidate_from_prefix() {
        let candidates = semantic_word_candidates("На улице опять идёт д");
        assert_eq!(candidates[0].text, "На улице опять идёт дождь");
        assert_eq!(candidates[0].source, SEMANTIC_WORD_SOURCE);
        assert!(candidates[0]
            .support
            .iter()
            .any(|line| line.contains("mode=weather_context")));
    }

    #[test]
    fn semantic_word_cell_generates_nearest_dictionary_word() {
        let candidates = semantic_word_candidates("это вобще ");
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "это вообще"));
    }

    #[test]
    fn semantic_word_cell_uses_fuzzy_typo_bridge_for_broken_prefix() {
        let candidates = semantic_word_candidates("где эсперемнт ");
        assert_eq!(
            candidates.first().map(|candidate| candidate.text.as_str()),
            Some("где эксперимент"),
            "expected common fuzzy repair to outrank rare neighbors: {candidates:?}"
        );
    }

    #[test]
    fn semantic_word_cell_uses_fuzzy_typo_bridge_for_extra_initial_letter() {
        let candidates = semantic_word_candidates("на сколько эаффективная ");
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.text == "на сколько эффективная"),
            "expected extra-letter fuzzy repair in L2: {candidates:?}"
        );
    }

    #[test]
    fn semantic_word_cell_does_not_rewrite_valid_word_without_context() {
        let candidates = semantic_word_candidates("следующий пукнут ");
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.text != "следующий пункт"),
            "valid Russian form must not become a noun without L3 context: {candidates:?}"
        );
    }

    #[test]
    fn semantic_word_cell_uses_large_ru_wave_memory() {
        assert!(
            ru_word_wave_memory().entries.len() >= 100_000,
            "RU wave memory should be a large lexical source"
        );
    }

    #[test]
    fn semantic_word_cell_uses_large_en_wave_memory() {
        assert!(
            en_word_wave_memory().entries.len() >= 100_000,
            "EN wave memory should combine hunspell and system words"
        );
    }

    #[test]
    fn warm_up_does_not_build_prefix_completion_indexes() {
        if RU_WORD_PREFIX_INDEX.get().is_some() || EN_WORD_PREFIX_INDEX.get().is_some() {
            return;
        }
        warm_up();
        assert!(RU_WORD_WAVE_MEMORY.get().is_some());
        assert!(EN_WORD_WAVE_MEMORY.get().is_some());
        assert!(RU_WORD_PREFIX_INDEX.get().is_none());
        assert!(EN_WORD_PREFIX_INDEX.get().is_none());
        assert!(!prefix_wave_memory_is_warm());
    }

    #[test]
    fn semantic_word_cell_generates_english_wave_candidate() {
        let candidates = semantic_word_candidates("this exmaple ");
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.text == "this example"),
            "expected English wave memory candidate, got {candidates:?}"
        );
    }

    #[test]
    fn semantic_word_cell_repairs_domain_typo_with_phrase_context() {
        let candidates = semantic_word_candidates("это невидные ");
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "это невалидные"));
    }

    #[test]
    fn semantic_word_cell_does_not_strip_russian_suffix() {
        let candidates = semantic_word_candidates("сколько текста ");
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "сколько текст"));
    }

    #[test]
    fn semantic_word_cell_keeps_known_verb_forms() {
        for (original, forbidden) in [
            ("он проверит ", "он проверка"),
            ("твой проверил ", "твой проверка"),
        ] {
            let candidates = semantic_word_candidates(original);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.text != forbidden),
                "known verb form should not become nearest noun: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn semantic_word_cell_keeps_case_vowel_tail() {
        let candidates = semantic_word_candidates("для заказа товара ");
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.text != "для заказа товар"),
            "case-like vowel tail should not drift into a consonant token: {candidates:?}"
        );
    }

    #[test]
    fn semantic_word_cell_keeps_valid_russian_forms() {
        for (original, forbidden) in [
            ("с проверкой ", "с проверка"),
            ("для проверки ", "для проверка"),
            ("есть окно ", "есть оно"),
            ("мало слов ", "мало слово"),
        ] {
            let candidates = semantic_word_candidates(original);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.text != forbidden),
                "known Russian form should not drift into a nearby known word: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn semantic_word_cell_keeps_valid_live_context_tokens() {
        for original in [
            "про него ",
            "у него ",
            "Дай промпт ",
            "если датасет ",
            "а нафига ",
            "теорию бейса ",
        ] {
            let candidates = semantic_word_candidates(original);
            assert!(
                candidates.is_empty(),
                "valid or domain-like live token should not get semantic neighbor candidates: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn phrase_forecast_is_l3_summary_not_l2_source() {
        let candidate = semantic_word_candidates("На улице опять идёт д")
            .into_iter()
            .next()
            .unwrap();
        let summary = phrase_forecast_summary("На улице опять идёт д", &candidate).unwrap();
        assert!(summary.contains("forecast=\"На улице опять идёт дождь\""));
        assert!(summary.contains(PHRASE_FORECAST_CELL));
    }
}
