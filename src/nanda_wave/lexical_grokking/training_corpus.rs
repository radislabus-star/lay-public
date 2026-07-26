use std::collections::HashMap;

use crate::stable_hash::mix64_golden;

const AMBIGUITY_BLOOM_BITS_PER_SURFACE: usize = 16;
const AMBIGUITY_BLOOM_HASHES: u64 = 3;

#[cfg(test)]
pub(super) struct TrainingWord {
    pub(super) terminal_id: u32,
    pub(super) surface: String,
    pub(super) training_surfaces: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
struct SurfaceSpan {
    start: u32,
    len: u32,
}

pub(super) struct TrainingCorpusWord {
    pub(super) terminal_id: u32,
    pub(super) surface: String,
    surface_start: u32,
    surface_count: u16,
}

pub(super) struct TrainingCorpus {
    words: Vec<TrainingCorpusWord>,
    spans: Vec<SurfaceSpan>,
    bytes: Vec<u8>,
}

impl TrainingCorpus {
    pub(super) fn try_with_capacity(
        word_capacity: usize,
        surface_capacity: usize,
        byte_capacity: usize,
    ) -> Result<Self, String> {
        let mut words = Vec::new();
        words
            .try_reserve_exact(word_capacity)
            .map_err(|error| format!("L1 training word allocation failed: {error}"))?;
        let mut spans = Vec::new();
        spans
            .try_reserve_exact(surface_capacity)
            .map_err(|error| format!("L1 training span allocation failed: {error}"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(byte_capacity)
            .map_err(|error| format!("L1 training byte arena allocation failed: {error}"))?;
        Ok(Self {
            words,
            spans,
            bytes,
        })
    }

    #[cfg(test)]
    pub(super) fn from_words(words: &[TrainingWord]) -> Result<Self, String> {
        let surface_capacity = words.iter().map(|word| word.training_surfaces.len()).sum();
        let byte_capacity = words
            .iter()
            .flat_map(|word| word.training_surfaces.iter())
            .map(String::len)
            .sum();
        let mut corpus = Self::try_with_capacity(words.len(), surface_capacity, byte_capacity)?;
        for word in words {
            corpus.push_word(
                word.terminal_id,
                word.surface.clone(),
                word.training_surfaces.iter().cloned(),
            )?;
        }
        Ok(corpus)
    }

    pub(super) fn push_word<I>(
        &mut self,
        terminal_id: u32,
        surface: String,
        training_surfaces: I,
    ) -> Result<(), String>
    where
        I: IntoIterator<Item = String>,
    {
        let expected_terminal = self
            .words
            .last()
            .map(|word| word.terminal_id.saturating_add(1))
            .unwrap_or(terminal_id);
        if terminal_id != expected_terminal {
            return Err("L1 crystal terminal IDs must be dense and ordered".to_string());
        }
        let surface_start = u32::try_from(self.spans.len())
            .map_err(|_| "L1 training surface count exceeds u32".to_string())?;
        let mut surface_count = 0_usize;
        for training_surface in training_surfaces {
            let start = u32::try_from(self.bytes.len())
                .map_err(|_| "L1 training byte arena exceeds 4 GiB".to_string())?;
            let len = u32::try_from(training_surface.len())
                .map_err(|_| "L1 training surface exceeds 4 GiB".to_string())?;
            self.bytes.extend_from_slice(training_surface.as_bytes());
            self.spans.push(SurfaceSpan { start, len });
            surface_count += 1;
        }
        self.words.push(TrainingCorpusWord {
            terminal_id,
            surface,
            surface_start,
            surface_count: u16::try_from(surface_count)
                .map_err(|_| "L1 training surfaces per word exceed u16".to_string())?,
        });
        Ok(())
    }

    pub(super) fn append_shard(&mut self, mut shard: Self) -> Result<(), String> {
        if shard.words.is_empty() {
            return Ok(());
        }
        let expected_terminal = u32::try_from(self.words.len())
            .map_err(|_| "L1 training word count exceeds u32".to_string())?;
        if shard.words[0].terminal_id != expected_terminal {
            return Err("L1 training shards must be appended in terminal order".to_string());
        }
        let span_offset = u32::try_from(self.spans.len())
            .map_err(|_| "L1 training surface count exceeds u32".to_string())?;
        let byte_offset = u32::try_from(self.bytes.len())
            .map_err(|_| "L1 training byte arena exceeds 4 GiB".to_string())?;
        for word in &mut shard.words {
            word.surface_start = word
                .surface_start
                .checked_add(span_offset)
                .ok_or_else(|| "L1 training surface offset exceeds u32".to_string())?;
        }
        for span in &mut shard.spans {
            span.start = span
                .start
                .checked_add(byte_offset)
                .ok_or_else(|| "L1 training byte offset exceeds u32".to_string())?;
        }
        self.words.append(&mut shard.words);
        self.spans.append(&mut shard.spans);
        self.bytes.append(&mut shard.bytes);
        Ok(())
    }

    pub(super) fn words(&self) -> &[TrainingCorpusWord] {
        &self.words
    }

    pub(super) fn training_surfaces<'a>(
        &'a self,
        word: &'a TrainingCorpusWord,
    ) -> impl Iterator<Item = &'a str> + 'a {
        let start = word.surface_start as usize;
        let end = start + word.surface_count as usize;
        self.spans[start..end]
            .iter()
            .map(|span| self.surface(*span))
    }

    pub(super) fn clean_surface<'a>(&self, word: &'a TrainingCorpusWord) -> &'a str {
        word.surface.as_str()
    }

    pub(super) fn word_surfaces<'a>(
        &'a self,
        word: &'a TrainingCorpusWord,
    ) -> impl Iterator<Item = &'a str> + 'a {
        std::iter::once(self.clean_surface(word)).chain(self.training_surfaces(word))
    }

    pub(super) fn training_surface_count(&self) -> usize {
        self.spans.len()
    }

    pub(super) fn packed_surface_bytes(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn span_bytes(&self) -> usize {
        self.spans.len() * std::mem::size_of::<SurfaceSpan>()
    }

    pub(super) fn ambiguous_surface_owners(&self) -> HashMap<&str, Vec<u32>> {
        if self.spans.is_empty() && self.words.is_empty() {
            return HashMap::new();
        }
        let bit_count = self
            .spans
            .len()
            .saturating_add(self.words.len())
            .saturating_mul(AMBIGUITY_BLOOM_BITS_PER_SURFACE)
            .max(64);
        let mut bloom = vec![0_u64; bit_count.div_ceil(64)];
        let mut possible_duplicates = HashMap::<&str, Vec<u32>>::new();

        for word in &self.words {
            for surface in self.word_surfaces(word) {
                if bloom_seen_and_insert(&mut bloom, surface) {
                    possible_duplicates.entry(surface).or_default();
                }
            }
        }
        drop(bloom);

        for word in &self.words {
            for surface in self.word_surfaces(word) {
                let Some(owners) = possible_duplicates.get_mut(surface) else {
                    continue;
                };
                if owners.last().copied() != Some(word.terminal_id) {
                    owners.push(word.terminal_id);
                }
            }
        }
        possible_duplicates.retain(|_, owners| owners.len() > 1);
        possible_duplicates.shrink_to_fit();
        possible_duplicates
    }

    fn surface(&self, span: SurfaceSpan) -> &str {
        let start = span.start as usize;
        let end = start + span.len as usize;
        std::str::from_utf8(&self.bytes[start..end])
            .expect("L1 training arena only stores validated UTF-8 strings")
    }
}

fn bloom_seen_and_insert(bits: &mut [u64], surface: &str) -> bool {
    let bit_count = bits.len() as u64 * 64;
    let (first, second) = surface_hashes(surface);
    let mut seen = true;
    for index in 0..AMBIGUITY_BLOOM_HASHES {
        let bit = first
            .wrapping_add(index.wrapping_mul(second))
            .wrapping_rem(bit_count);
        let word = bit as usize / 64;
        let mask = 1_u64 << (bit % 64);
        seen &= bits[word] & mask != 0;
        bits[word] |= mask;
    }
    seen
}

fn surface_hashes(surface: &str) -> (u64, u64) {
    let mut first = 0x4c31_315f_5355_5246_u64;
    let mut second = 0x4259_5445_4152_454e_u64;
    for byte in surface.bytes() {
        first = mix64_golden(first ^ u64::from(byte));
        second = mix64_golden(second ^ u64::from(byte).rotate_left(17));
    }
    (first, second | 1)
}
