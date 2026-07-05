//! Canonical L1 center memory shadow path.
//!
//! L1 stores reusable 4-gram/position surface centers and per-token center-id
//! sequences. It does not store whole words as authority.

use std::collections::HashMap;

use super::mode::mix64_golden;
use super::surface_wave::{
    surface_atom_projection, surface_atoms, SurfaceWaveTrit, SURFACE_WAVE_TRITS,
};

const L1_CENTER_RECORD_BYTES: usize = 32;
pub(super) const L1_SEQUENCE_REF_BYTES: usize = 4;
const L1_WORD_RECORD_BYTES: usize = 16;
const L1_RESIDUAL_NGRAM_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct L1CenterMemoryConfig {
    pub(super) min_center_support: usize,
    pub(super) max_centers: usize,
}

impl Default for L1CenterMemoryConfig {
    fn default() -> Self {
        Self {
            min_center_support: 2,
            max_centers: 1_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct L1CenterKey {
    ngram_hash: u64,
    position_code: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct L1SurfaceCenter {
    id: u32,
    ngram_hash: u64,
    position_code: u16,
    support: u32,
    trits: [SurfaceWaveTrit; SURFACE_WAVE_TRITS],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct L1WordCenterRecord {
    source_hash: u64,
    ngram_count: u16,
    center_ref_start: u32,
    center_ref_len: u16,
    residual_ngram_count: u16,
}

#[derive(Clone, Debug)]
pub(super) struct L1CenterMemory {
    centers: Vec<L1SurfaceCenter>,
    center_index: HashMap<L1CenterKey, u32>,
    word_records: Vec<L1WordCenterRecord>,
    sequence_refs: Vec<u32>,
    residual_ngram_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct L1CenterSequence {
    pub(super) ngram_count: usize,
    pub(super) center_refs: Vec<u32>,
    pub(super) residual_ngrams: usize,
}

impl L1CenterMemory {
    #[must_use]
    pub(super) fn build<'a, I>(words: I, config: L1CenterMemoryConfig) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let words = words.into_iter().map(str::to_string).collect::<Vec<_>>();
        let centers = build_centers(words.iter().map(String::as_str), config);
        let center_index = centers
            .iter()
            .map(|center| {
                (
                    L1CenterKey {
                        ngram_hash: center.ngram_hash,
                        position_code: center.position_code,
                    },
                    center.id,
                )
            })
            .collect::<HashMap<_, _>>();

        let mut memory = Self {
            centers,
            center_index,
            word_records: Vec::with_capacity(words.len()),
            sequence_refs: Vec::new(),
            residual_ngram_count: 0,
        };
        for word in &words {
            memory.encode_train_word(word);
        }
        memory
    }

    #[must_use]
    pub(super) fn center_count(&self) -> usize {
        self.centers.len()
    }

    #[must_use]
    pub(super) fn word_records(&self) -> &[L1WordCenterRecord] {
        &self.word_records
    }

    #[must_use]
    pub(super) fn sequence_refs(&self) -> &[u32] {
        &self.sequence_refs
    }

    #[must_use]
    pub(super) fn hot_bytes(&self) -> usize {
        self.centers.len() * L1_CENTER_RECORD_BYTES
            + self.sequence_refs.len() * L1_SEQUENCE_REF_BYTES
            + self.word_records.len() * L1_WORD_RECORD_BYTES
            + self.residual_ngram_count * L1_RESIDUAL_NGRAM_BYTES
    }

    #[must_use]
    pub(super) fn center_sequence_for_word(&self, word: &str) -> L1CenterSequence {
        let refs = self.center_refs_for_word(word);
        L1CenterSequence {
            ngram_count: refs.ngram_count,
            center_refs: refs.center_refs,
            residual_ngrams: refs.residual_ngrams,
        }
    }

    fn encode_train_word(&mut self, word: &str) {
        let refs = self.center_refs_for_word(word);
        let start = self.sequence_refs.len();
        self.sequence_refs.extend(refs.center_refs.iter().copied());
        self.residual_ngram_count += refs.residual_ngrams;
        self.word_records.push(L1WordCenterRecord {
            source_hash: stable_hash(word.as_bytes()),
            ngram_count: refs.ngram_count.min(u16::MAX as usize) as u16,
            center_ref_start: start.min(u32::MAX as usize) as u32,
            center_ref_len: refs.center_refs.len().min(u16::MAX as usize) as u16,
            residual_ngram_count: refs.residual_ngrams.min(u16::MAX as usize) as u16,
        });
    }

    fn center_refs_for_word(&self, word: &str) -> WordRefs {
        let mut refs = WordRefs::default();
        for atom in surface_atoms(word) {
            refs.ngram_count += 1;
            let key = L1CenterKey {
                ngram_hash: stable_hash(&atom.bytes),
                position_code: position_code(atom.position),
            };
            if let Some(center_id) = self.center_index.get(&key) {
                refs.center_refs.push(*center_id);
            } else {
                refs.residual_ngrams += 1;
            }
        }
        refs
    }
}

#[derive(Default)]
struct WordRefs {
    ngram_count: usize,
    center_refs: Vec<u32>,
    residual_ngrams: usize,
}

#[derive(Clone, Copy, Debug)]
struct CenterStats {
    support: usize,
    trits: [SurfaceWaveTrit; SURFACE_WAVE_TRITS],
}

fn build_centers<'a, I>(words: I, config: L1CenterMemoryConfig) -> Vec<L1SurfaceCenter>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut candidates: HashMap<L1CenterKey, CenterStats> = HashMap::new();
    for word in words {
        for atom in surface_atoms(word) {
            let trits = surface_atom_projection(atom.position, &atom.bytes);
            candidates
                .entry(L1CenterKey {
                    ngram_hash: stable_hash(&atom.bytes),
                    position_code: position_code(atom.position),
                })
                .and_modify(|stats| stats.support += 1)
                .or_insert(CenterStats { support: 1, trits });
        }
    }

    let mut centers = candidates
        .into_iter()
        .filter(|(_, stats)| stats.support >= config.min_center_support)
        .collect::<Vec<_>>();
    centers.sort_by(|(left_key, left), (right_key, right)| {
        right
            .support
            .cmp(&left.support)
            .then_with(|| left_key.ngram_hash.cmp(&right_key.ngram_hash))
            .then_with(|| left_key.position_code.cmp(&right_key.position_code))
    });
    centers.truncate(config.max_centers);

    centers
        .into_iter()
        .enumerate()
        .map(|(id, (key, stats))| L1SurfaceCenter {
            id: id as u32,
            ngram_hash: key.ngram_hash,
            position_code: key.position_code,
            support: stats.support as u32,
            trits: stats.trits,
        })
        .collect()
}

fn position_code(position: u64) -> u16 {
    let low = (position as u16) & 0x3f;
    let block = ((position / 8) as u16) & 0x03ff;
    low | (block << 6)
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut state = 0x4C31_4345_4E54_4552u64;
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
    fn l1_center_memory_stores_sequences_not_whole_word_waves() {
        let words = [
            "волновая",
            "волнового",
            "волновому",
            "памятная",
            "памятного",
            "памятному",
        ];
        let memory = L1CenterMemory::build(
            words.iter().copied(),
            L1CenterMemoryConfig {
                min_center_support: 1,
                ..L1CenterMemoryConfig::default()
            },
        );

        assert!(memory.center_count() > 0);
        assert_eq!(memory.word_records().len(), words.len());
        assert!(memory.sequence_refs().len() > words.len());
        assert!(memory.hot_bytes() < words.len() * super::super::surface_wave::SURFACE_WAVE_BYTES);

        let sequence = memory.center_sequence_for_word("волновыми");
        assert!(sequence.ngram_count > 0);
        assert!(!sequence.center_refs.is_empty());
    }

    #[test]
    fn l1_center_memory_keeps_short_words_and_function_words_as_surface_atoms() {
        let words = ["и", "в", "не", "сыч", "и", "в", "не", "сыч", "не работает"];
        let memory = L1CenterMemory::build(
            words.iter().copied(),
            L1CenterMemoryConfig {
                min_center_support: 1,
                ..L1CenterMemoryConfig::default()
            },
        );

        for word in ["и", "в", "не", "сыч"] {
            let sequence = memory.center_sequence_for_word(word);
            assert!(
                sequence.ngram_count > 0,
                "word={word} sequence={sequence:?}"
            );
            assert!(!sequence.center_refs.is_empty(), "word={word}");
        }

        let service = memory.center_sequence_for_word("и");
        let content = memory.center_sequence_for_word("сыч");
        assert_ne!(service.center_refs, content.center_refs);
    }
}
