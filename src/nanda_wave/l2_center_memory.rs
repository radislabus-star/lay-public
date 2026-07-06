//! Canonical L2 center memory shadow path.
//!
//! L2 consumes L1 center-id sequences and promotes repeated sequence motifs.

use std::collections::{HashMap, HashSet};

use crate::text_metrics::damerau_levenshtein;

use super::l1_center_memory::{L1CenterMemory, L1CenterMemoryConfig, L1_SEQUENCE_REF_BYTES};
use super::mode::mix64_golden;

const L2_CENTER_RECORD_BYTES: usize = 32;
const L2_TOKEN_REF_BYTES: usize = 4;
const L2_WORD_RECORD_BYTES: usize = 16;
const L2_RESIDUAL_REF_BYTES: usize = 4;
const L2_RESIDUAL_TAG: u32 = 1 << 31;
const MAX_LENGTH_BUCKET_CANDIDATES: usize = 1536;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct L2CenterMemoryConfig {
    pub(super) l1_config: L1CenterMemoryConfig,
    pub(super) motif_len: usize,
    pub(super) min_motif_support: usize,
    pub(super) max_motifs: usize,
}

impl Default for L2CenterMemoryConfig {
    fn default() -> Self {
        Self {
            l1_config: L1CenterMemoryConfig::default(),
            motif_len: 4,
            min_motif_support: 4,
            max_motifs: 512_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct L2SequenceCenter {
    id: u32,
    sequence_hash: u64,
    support: u32,
    l1_center_refs: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct L2WordRecord {
    source_hash: u64,
    l1_ref_count: u16,
    token_start: u32,
    token_len: u16,
    covered_l1_refs: u16,
    residual_l1_refs: u16,
}

#[derive(Clone, Debug)]
pub(super) struct L2CenterMemory {
    l1: L1CenterMemory,
    centers: Vec<L2SequenceCenter>,
    max_center_len: usize,
    center_index: HashMap<Vec<u32>, u32>,
    source_words: Vec<String>,
    word_records: Vec<L2WordRecord>,
    token_refs: Vec<u32>,
    token_to_words: HashMap<u32, Vec<usize>>,
    length_to_words: HashMap<usize, Vec<usize>>,
    residual_ref_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct L2TokenSequence {
    pub(super) l1_ref_count: usize,
    pub(super) tokens: Vec<u32>,
    pub(super) motif_refs: usize,
    pub(super) covered_l1_refs: usize,
    pub(super) residual_l1_refs: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct L2SurfaceCandidate {
    pub(super) word: String,
    pub(super) score: u32,
    pub(super) l1_overlap: usize,
    pub(super) l2_overlap: usize,
    pub(super) motif_overlap: usize,
    pub(super) prefix_match: bool,
}

impl L2CenterMemory {
    #[must_use]
    pub(super) fn build<'a, I>(words: I, config: L2CenterMemoryConfig) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let words = words.into_iter().map(str::to_string).collect::<Vec<_>>();
        let l1 = L1CenterMemory::build(words.iter().map(String::as_str), config.l1_config);
        let train_sequences = words
            .iter()
            .map(|word| l1.center_sequence_for_word(word).center_refs)
            .collect::<Vec<_>>();
        let centers = build_l2_centers(&train_sequences, config);
        let max_center_len = centers
            .iter()
            .map(|center| center.l1_center_refs.len())
            .max()
            .unwrap_or(0);
        let center_index = centers
            .iter()
            .map(|center| (center.l1_center_refs.clone(), center.id))
            .collect::<HashMap<_, _>>();
        let mut memory = Self {
            l1,
            centers,
            max_center_len,
            center_index,
            source_words: words.clone(),
            word_records: Vec::with_capacity(words.len()),
            token_refs: Vec::new(),
            token_to_words: HashMap::new(),
            length_to_words: HashMap::new(),
            residual_ref_count: 0,
        };

        for (word_index, word) in words.iter().enumerate() {
            memory.encode_train_word(word_index, word);
        }
        memory.sort_runtime_indexes();

        memory
    }

    #[must_use]
    pub(super) fn l1(&self) -> &L1CenterMemory {
        &self.l1
    }

    #[must_use]
    pub(super) fn center_count(&self) -> usize {
        self.centers.len()
    }

    #[must_use]
    pub(super) fn source_word_count(&self) -> usize {
        self.source_words.len()
    }

    #[must_use]
    pub(super) fn word_records(&self) -> &[L2WordRecord] {
        &self.word_records
    }

    #[must_use]
    pub(super) fn token_refs(&self) -> &[u32] {
        &self.token_refs
    }

    #[must_use]
    pub(super) fn token_sequence_for_text(&self, text: &str) -> L2TokenSequence {
        let sequence = self.l1.center_sequence_for_word(text).center_refs;
        let encoded = self.encode_sequence(&sequence);
        L2TokenSequence {
            l1_ref_count: sequence.len(),
            tokens: encoded.tokens,
            motif_refs: encoded.motif_refs,
            covered_l1_refs: encoded.covered_l1_refs,
            residual_l1_refs: encoded.residual_l1_refs,
        }
    }

    #[must_use]
    pub(super) fn hot_bytes(&self) -> usize {
        self.centers.len() * L2_CENTER_RECORD_BYTES
            + self
                .centers
                .iter()
                .map(|center| center.l1_center_refs.len() * L1_SEQUENCE_REF_BYTES)
                .sum::<usize>()
            + self.token_refs.len() * L2_TOKEN_REF_BYTES
            + self.word_records.len() * L2_WORD_RECORD_BYTES
            + self.residual_ref_count * L2_RESIDUAL_REF_BYTES
    }

    #[must_use]
    pub(super) fn surface_candidates_for_text(
        &self,
        text: &str,
        limit: usize,
    ) -> Vec<L2SurfaceCandidate> {
        let usage = super::usage_prior::cached_usage_prior_snapshot();
        self.surface_candidates_for_text_with_usage(text, limit, &usage)
    }

    #[must_use]
    pub(super) fn surface_candidates_for_text_with_usage(
        &self,
        text: &str,
        limit: usize,
        usage: &super::usage_prior::UsagePriorSnapshot,
    ) -> Vec<L2SurfaceCandidate> {
        if limit == 0 {
            return Vec::new();
        }
        let input_norm = normalize_surface(text);
        if input_norm.is_empty() {
            return Vec::new();
        }

        let query_l1 = self.l1.center_sequence_for_word(&input_norm);
        let query_l2 = self.token_sequence_for_text(&input_norm);
        if query_l1.center_refs.is_empty() && query_l2.tokens.is_empty() {
            return Vec::new();
        }

        let mut word_votes: HashMap<usize, u16> = HashMap::new();
        for token in unique_tokens(&query_l2.tokens) {
            let Some(word_ids) = self.token_to_words.get(&token) else {
                continue;
            };
            for word_id in word_ids.iter().copied().take(512) {
                *word_votes.entry(word_id).or_default() += 1;
            }
        }
        let mut candidate_ids = word_votes.into_keys().collect::<HashSet<_>>();
        let input_len = input_len_for_bucket(&input_norm);
        if input_len >= 4 {
            let max_gap = if input_len >= 8 { 3 } else { 2 };
            let cap = limit.saturating_mul(128).max(MAX_LENGTH_BUCKET_CANDIDATES);
            for word_id in self.length_bucket_word_ids(input_len, max_gap, cap) {
                candidate_ids.insert(word_id);
            }
        }

        let mut candidates = candidate_ids
            .into_iter()
            .filter_map(|word_id| {
                let word = self.source_words.get(word_id)?;
                let word_norm = normalize_surface(word);
                if word_norm.is_empty() || word_norm == input_norm {
                    return None;
                }
                let input_len = input_norm.chars().count();
                let word_len = word_norm.chars().count();
                if input_len.abs_diff(word_len) > 4 {
                    return None;
                }

                let word_l1_refs = self.l1.center_refs_for_record(word_id);
                let word_l2_tokens = self.token_refs_for_record(word_id);
                let l1_overlap = overlap_count(&query_l1.center_refs, word_l1_refs);
                let l2_overlap = overlap_count(&query_l2.tokens, word_l2_tokens);
                let motif_overlap = overlap_count(
                    &motif_tokens(&query_l2.tokens),
                    &motif_tokens(word_l2_tokens),
                );
                let surface_distance = damerau_levenshtein(&input_norm, &word_norm);
                let prefix_match =
                    word_norm.starts_with(&input_norm) || input_norm.starts_with(&word_norm);
                let mut score = candidate_score(
                    input_len,
                    word_len,
                    surface_distance,
                    l1_overlap,
                    l2_overlap,
                    motif_overlap,
                    prefix_match,
                );
                if score > 0 {
                    score = score.saturating_add(usage_score_boost(usage, &word_norm));
                }
                (score > 0).then_some(L2SurfaceCandidate {
                    word: word_norm,
                    score,
                    l1_overlap,
                    l2_overlap,
                    motif_overlap,
                    prefix_match,
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| right.motif_overlap.cmp(&left.motif_overlap))
                .then_with(|| right.l2_overlap.cmp(&left.l2_overlap))
                .then_with(|| right.l1_overlap.cmp(&left.l1_overlap))
                .then_with(|| left.word.len().cmp(&right.word.len()))
                .then_with(|| left.word.cmp(&right.word))
        });
        candidates.dedup_by(|left, right| left.word == right.word);
        candidates.truncate(limit);
        candidates
    }

    fn encode_train_word(&mut self, word_index: usize, word: &str) {
        let sequence = self.l1.center_sequence_for_word(word).center_refs;
        let encoded = self.encode_sequence(&sequence);
        let start = self.token_refs.len();
        self.token_refs.extend(encoded.tokens.iter().copied());
        for token in unique_tokens(&encoded.tokens) {
            self.token_to_words
                .entry(token)
                .or_default()
                .push(word_index);
        }
        let word_len = normalize_surface(word).chars().count();
        self.length_to_words
            .entry(word_len)
            .or_default()
            .push(word_index);
        self.residual_ref_count += encoded.residual_l1_refs;
        self.word_records.push(L2WordRecord {
            source_hash: stable_hash_bytes(word.as_bytes()),
            l1_ref_count: sequence.len().min(u16::MAX as usize) as u16,
            token_start: start.min(u32::MAX as usize) as u32,
            token_len: encoded.tokens.len().min(u16::MAX as usize) as u16,
            covered_l1_refs: encoded.covered_l1_refs.min(u16::MAX as usize) as u16,
            residual_l1_refs: encoded.residual_l1_refs.min(u16::MAX as usize) as u16,
        });
    }

    fn token_refs_for_record(&self, word_index: usize) -> &[u32] {
        let Some(record) = self.word_records.get(word_index) else {
            return &[];
        };
        let start = record.token_start as usize;
        let end = start.saturating_add(record.token_len as usize);
        self.token_refs.get(start..end).unwrap_or(&[])
    }

    fn sort_runtime_indexes(&mut self) {
        let usage = super::usage_prior::cached_usage_prior_snapshot();
        let priorities = self
            .source_words
            .iter()
            .map(|word| WordRuntimePriority {
                usage: usage.word_prior(word),
                common: crate::lexicon::is_common_ru_word(word),
                len: word.chars().count(),
            })
            .collect::<Vec<_>>();
        for ids in self.token_to_words.values_mut() {
            sort_word_ids_by_runtime_priority(&self.source_words, &priorities, ids);
        }
        for ids in self.length_to_words.values_mut() {
            sort_word_ids_by_runtime_priority(&self.source_words, &priorities, ids);
        }
    }

    fn length_bucket_word_ids(&self, input_len: usize, max_gap: usize, cap: usize) -> Vec<usize> {
        let mut ids = Vec::new();
        let start = input_len.saturating_sub(max_gap);
        let end = input_len + max_gap;
        for len in start..=end {
            if let Some(word_ids) = self.length_to_words.get(&len) {
                let remaining = cap.saturating_sub(ids.len());
                if remaining == 0 {
                    break;
                }
                ids.extend(word_ids.iter().copied().take(remaining));
            }
        }
        ids
    }

    fn encode_sequence(&self, sequence: &[u32]) -> EncodedSequence {
        if sequence.is_empty() {
            return EncodedSequence::default();
        }

        let mut encoded = EncodedSequence::default();
        let mut index = 0usize;
        while index < sequence.len() {
            let remaining = sequence.len() - index;
            if remaining >= 4 {
                let max_len = self.max_center_len.min(remaining);
                let mut matched = None;
                for len in (1..=max_len).rev() {
                    if let Some(id) = self.center_index.get(&sequence[index..index + len]) {
                        matched = Some((*id, len));
                        break;
                    }
                }
                if let Some((id, len)) = matched {
                    encoded.tokens.push(id);
                    encoded.motif_refs += 1;
                    encoded.covered_l1_refs += len;
                    index += len;
                    continue;
                }
            }

            encoded.tokens.push(tag_residual_token(sequence[index]));
            encoded.residual_l1_refs += 1;
            index += 1;
        }
        encoded
    }
}

fn input_len_for_bucket(text: &str) -> usize {
    text.chars().count()
}

#[derive(Clone, Debug, Default)]
struct EncodedSequence {
    tokens: Vec<u32>,
    covered_l1_refs: usize,
    residual_l1_refs: usize,
    motif_refs: usize,
}

fn build_l2_centers(sequences: &[Vec<u32>], config: L2CenterMemoryConfig) -> Vec<L2SequenceCenter> {
    let mut candidates: HashMap<Vec<u32>, usize> = HashMap::new();
    if config.motif_len == 0 {
        return Vec::new();
    }

    for sequence in sequences {
        if sequence.len() < config.motif_len {
            continue;
        }
        for window in sequence.windows(config.motif_len) {
            *candidates.entry(window.to_vec()).or_default() += 1;
        }
    }

    let mut centers = candidates
        .into_iter()
        .filter(|(_, support)| *support >= config.min_motif_support)
        .collect::<Vec<_>>();
    centers.sort_by(
        |(left_sequence, left_support), (right_sequence, right_support)| {
            right_support
                .cmp(left_support)
                .then_with(|| left_sequence.cmp(right_sequence))
        },
    );
    centers.truncate(config.max_motifs);

    centers
        .into_iter()
        .enumerate()
        .map(|(id, (sequence, support))| L2SequenceCenter {
            id: id as u32,
            sequence_hash: stable_hash_u32s(&sequence),
            support: support as u32,
            l1_center_refs: sequence,
        })
        .collect()
}

fn tag_residual_token(center_ref: u32) -> u32 {
    center_ref | L2_RESIDUAL_TAG
}

fn unique_tokens(tokens: &[u32]) -> Vec<u32> {
    let mut tokens = tokens.to_vec();
    tokens.sort_unstable();
    tokens.dedup();
    tokens
}

fn motif_tokens(tokens: &[u32]) -> Vec<u32> {
    tokens
        .iter()
        .copied()
        .filter(|token| token & L2_RESIDUAL_TAG == 0)
        .collect()
}

fn overlap_count(left: &[u32], right: &[u32]) -> usize {
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    left.sort_unstable();
    right.sort_unstable();
    let mut count = 0usize;
    let mut li = 0usize;
    let mut ri = 0usize;
    while li < left.len() && ri < right.len() {
        match left[li].cmp(&right[ri]) {
            std::cmp::Ordering::Less => li += 1,
            std::cmp::Ordering::Greater => ri += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                li += 1;
                ri += 1;
            }
        }
    }
    count
}

fn candidate_score(
    input_len: usize,
    word_len: usize,
    surface_distance: usize,
    l1_overlap: usize,
    l2_overlap: usize,
    motif_overlap: usize,
    prefix_match: bool,
) -> u32 {
    if input_len == 0 {
        return 0;
    }
    let len_gap = input_len.abs_diff(word_len).min(16) as u32;
    let mut score = l1_overlap as u32 * 24 + l2_overlap as u32 * 48 + motif_overlap as u32 * 120;
    if input_len >= 5 {
        let close_distance = if input_len >= 6 { 3 } else { 2 };
        if surface_distance <= close_distance {
            score += 1_380u32.saturating_sub(surface_distance as u32 * 260);
        } else if !prefix_match && motif_overlap == 0 && l2_overlap < 3 {
            return 0;
        }
    }
    if prefix_match {
        score += 180;
    }
    score.saturating_sub(len_gap * 8 + surface_distance.min(16) as u32 * 12)
}

fn usage_score_boost(usage: &super::usage_prior::UsagePriorSnapshot, word: &str) -> u32 {
    let prior = usage.word_prior(word);
    (prior * 2_000.0).round().clamp(0.0, 220.0) as u32
}

#[derive(Clone, Copy, Debug)]
struct WordRuntimePriority {
    usage: f32,
    common: bool,
    len: usize,
}

fn sort_word_ids_by_runtime_priority(
    source_words: &[String],
    priorities: &[WordRuntimePriority],
    ids: &mut [usize],
) {
    ids.sort_by(|left_id, right_id| {
        let left = source_words
            .get(*left_id)
            .map(String::as_str)
            .unwrap_or_default();
        let right = source_words
            .get(*right_id)
            .map(String::as_str)
            .unwrap_or_default();
        let left_priority = priorities.get(*left_id).copied().unwrap_or_default();
        let right_priority = priorities.get(*right_id).copied().unwrap_or_default();
        right_priority
            .usage
            .total_cmp(&left_priority.usage)
            .then_with(|| right_priority.common.cmp(&left_priority.common))
            .then_with(|| left_priority.len.cmp(&right_priority.len))
            .then_with(|| left.cmp(right))
    });
}

impl Default for WordRuntimePriority {
    fn default() -> Self {
        Self {
            usage: 0.0,
            common: false,
            len: usize::MAX,
        }
    }
}

fn normalize_surface(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_alphabetic() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn stable_hash_u32s(values: &[u32]) -> u64 {
    let mut state = 0x4C32_5345_5155_454Eu64;
    for value in values {
        state ^= u64::from(*value).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        state = mix64_golden(state);
    }
    mix64_golden(state ^ values.len() as u64)
}

fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    let mut state = 0x4C32_574F_5244_0001u64;
    for byte in bytes {
        state ^= u64::from(*byte).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        state = mix64_golden(state);
    }
    mix64_golden(state ^ bytes.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l2_center_memory_builds_motifs_over_l1_sequences() {
        let words = [
            "проверка",
            "проверки",
            "проверкой",
            "проверяем",
            "проверял",
            "переворот",
            "перевороты",
            "переворачивать",
        ];
        let memory = L2CenterMemory::build(
            words.iter().copied(),
            L2CenterMemoryConfig {
                l1_config: L1CenterMemoryConfig {
                    min_center_support: 1,
                    ..L1CenterMemoryConfig::default()
                },
                motif_len: 3,
                min_motif_support: 2,
                ..L2CenterMemoryConfig::default()
            },
        );

        assert!(memory.l1().center_count() > 0);
        assert!(memory.center_count() > 0);
        assert!(memory.hot_bytes() > 0);

        let sequence = memory.token_sequence_for_text("проверочную");
        assert!(sequence.l1_ref_count > 0);
        assert!(sequence.motif_refs > 0, "sequence={sequence:?}");

        let candidates = memory.surface_candidates_for_text("проверк", 3);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "проверка"),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_center_memory_keeps_short_service_words_visible() {
        let words = ["и", "в", "не", "и", "в", "не", "и тест", "в тест"];
        let memory = L2CenterMemory::build(
            words.iter().copied(),
            L2CenterMemoryConfig {
                l1_config: L1CenterMemoryConfig {
                    min_center_support: 1,
                    ..L1CenterMemoryConfig::default()
                },
                motif_len: 1,
                min_motif_support: 1,
                ..L2CenterMemoryConfig::default()
            },
        );

        for word in ["и", "в", "не"] {
            let sequence = memory.token_sequence_for_text(word);
            assert!(sequence.l1_ref_count > 0, "word={word}");
            assert!(!sequence.tokens.is_empty(), "word={word}");
        }
    }

    #[test]
    fn l2_center_memory_ranks_surface_typo_candidates() {
        let words = [
            "загрузи",
            "загрузить",
            "выгрузи",
            "пункт",
            "пункты",
            "комитет",
            "подготовят",
        ];
        let memory = L2CenterMemory::build(
            words.iter().copied(),
            L2CenterMemoryConfig {
                l1_config: L1CenterMemoryConfig {
                    min_center_support: 1,
                    ..L1CenterMemoryConfig::default()
                },
                motif_len: 2,
                min_motif_support: 1,
                ..L2CenterMemoryConfig::default()
            },
        );

        let candidates = memory.surface_candidates_for_text("звгрузи", 3);
        assert_eq!(
            candidates.first().map(|candidate| candidate.word.as_str()),
            Some("загрузи"),
            "candidates={candidates:?}"
        );

        let candidates = memory.surface_candidates_for_text("коммит", 3);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "комитет"),
            "candidates={candidates:?}"
        );

        let candidates = memory.surface_candidates_for_text("подготовет", 3);
        assert_eq!(
            candidates.first().map(|candidate| candidate.word.as_str()),
            Some("подготовят"),
            "candidates={candidates:?}"
        );
    }
}
