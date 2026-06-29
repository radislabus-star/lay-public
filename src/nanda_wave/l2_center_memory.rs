//! Canonical L2 center memory shadow path.
//!
//! L2 consumes L1 center-id sequences and promotes repeated sequence motifs.

use std::collections::HashMap;

use super::l1_center_memory::{L1CenterMemory, L1CenterMemoryConfig, L1_SEQUENCE_REF_BYTES};

const L2_CENTER_RECORD_BYTES: usize = 32;
const L2_TOKEN_REF_BYTES: usize = 4;
const L2_WORD_RECORD_BYTES: usize = 16;
const L2_RESIDUAL_REF_BYTES: usize = 4;
const L2_RESIDUAL_TAG: u32 = 1 << 31;

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
    center_index: HashMap<Vec<u32>, u32>,
    word_records: Vec<L2WordRecord>,
    token_refs: Vec<u32>,
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
        let center_index = centers
            .iter()
            .map(|center| (center.l1_center_refs.clone(), center.id))
            .collect::<HashMap<_, _>>();
        let mut memory = Self {
            l1,
            centers,
            center_index,
            word_records: Vec::with_capacity(words.len()),
            token_refs: Vec::new(),
            residual_ref_count: 0,
        };

        for word in &words {
            memory.encode_train_word(word);
        }

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

    fn encode_train_word(&mut self, word: &str) {
        let sequence = self.l1.center_sequence_for_word(word).center_refs;
        let encoded = self.encode_sequence(&sequence);
        let start = self.token_refs.len();
        self.token_refs.extend(encoded.tokens.iter().copied());
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

    fn encode_sequence(&self, sequence: &[u32]) -> EncodedSequence {
        if sequence.is_empty() {
            return EncodedSequence::default();
        }

        let mut encoded = EncodedSequence::default();
        let mut index = 0usize;
        while index < sequence.len() {
            let remaining = sequence.len() - index;
            if remaining >= 4 {
                let max_len = self
                    .centers
                    .iter()
                    .map(|center| center.l1_center_refs.len())
                    .max()
                    .unwrap_or(0)
                    .min(remaining);
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

fn stable_hash_u32s(values: &[u32]) -> u64 {
    let mut state = 0x4C32_5345_5155_454Eu64;
    for value in values {
        state ^= u64::from(*value).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        state = mix64(state);
    }
    mix64(state ^ values.len() as u64)
}

fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    let mut state = 0x4C32_574F_5244_0001u64;
    for byte in bytes {
        state ^= u64::from(*byte).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        state = mix64(state);
    }
    mix64(state ^ bytes.len() as u64)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
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
}
