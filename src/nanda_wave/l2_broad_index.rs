//! Broad L2 surface readout over static lexical banks.
//!
//! This is not the L2 center heap. It keeps compact prefix routes into the
//! static word banks so live IME can reach a much wider source without scanning
//! the whole corpus or promoting every surface into hot center memory.

use std::collections::{HashMap, HashSet};

use crate::keyboard::is_cyrillic_letter;

const MIN_PREFIX_CHARS: usize = 2;
const MAX_PREFIX_CHARS: usize = 5;

#[derive(Debug)]
pub(super) struct L2BroadPrefixIndex {
    words: Vec<&'static str>,
    prefix_to_word_ids: HashMap<u128, Vec<u32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct L2BroadPrefixIndexStats {
    pub(super) source_words: usize,
    pub(super) prefix_keys: usize,
    pub(super) word_refs: usize,
}

impl L2BroadPrefixIndex {
    pub(super) fn build(sources: &[&'static str]) -> Self {
        let mut words = Vec::<&'static str>::new();
        let mut seen = HashSet::<&'static str>::new();
        for source in sources {
            for word in source.lines().filter_map(static_surface_word) {
                if seen.insert(word) {
                    words.push(word);
                }
            }
        }

        let mut prefix_to_word_ids = HashMap::<u128, Vec<u32>>::new();
        for (word_id, word) in words.iter().enumerate() {
            let len = word.chars().count();
            let max_prefix = MAX_PREFIX_CHARS.min(len.saturating_sub(1));
            for prefix_len in MIN_PREFIX_CHARS..=max_prefix {
                let Some(prefix) = prefix_key(word, prefix_len) else {
                    continue;
                };
                prefix_to_word_ids
                    .entry(prefix)
                    .or_default()
                    .push(word_id as u32);
            }
        }

        Self {
            words,
            prefix_to_word_ids,
        }
    }

    pub(super) fn prefix_candidates(
        &self,
        prefix: &str,
        min_chars: usize,
        max_chars: usize,
        limit: usize,
    ) -> Vec<&'static str> {
        if limit == 0 {
            return Vec::new();
        }
        let prefix_len = prefix.chars().count();
        if prefix_len < MIN_PREFIX_CHARS {
            return Vec::new();
        }
        let key_len = prefix_len.min(MAX_PREFIX_CHARS);
        let Some(key) = prefix_key(prefix, key_len) else {
            return Vec::new();
        };
        let Some(word_ids) = self.prefix_to_word_ids.get(&key) else {
            return Vec::new();
        };

        let mut out = Vec::with_capacity(limit.min(32));
        for word_id in word_ids {
            let Some(word) = self.words.get(*word_id as usize).copied() else {
                continue;
            };
            if !word.starts_with(prefix) {
                continue;
            }
            let len = word.chars().count();
            if !(min_chars..=max_chars).contains(&len) {
                continue;
            }
            out.push(word);
            if out.len() >= limit {
                break;
            }
        }
        out
    }

    pub(super) fn stats(&self) -> L2BroadPrefixIndexStats {
        L2BroadPrefixIndexStats {
            source_words: self.words.len(),
            prefix_keys: self.prefix_to_word_ids.len(),
            word_refs: self
                .prefix_to_word_ids
                .values()
                .map(std::vec::Vec::len)
                .sum(),
        }
    }
}

fn prefix_key(text: &str, prefix_len: usize) -> Option<u128> {
    if !(MIN_PREFIX_CHARS..=MAX_PREFIX_CHARS).contains(&prefix_len) {
        return None;
    }
    let mut key = (prefix_len as u128) << 120;
    let mut count = 0usize;
    for (idx, ch) in text.chars().take(prefix_len).enumerate() {
        key |= (ch as u32 as u128) << (idx * 21);
        count += 1;
    }
    (count == prefix_len).then_some(key)
}

fn static_surface_word(line: &'static str) -> Option<&'static str> {
    let word = line.trim();
    if word.is_empty() || word.starts_with('#') {
        return None;
    }
    let len = word.chars().count();
    if !(4..=24).contains(&len) {
        return None;
    }
    if !word.chars().all(is_cyrillic_letter) {
        return None;
    }
    if crate::lexicon::is_ru_live_protected_word(word) {
        return None;
    }
    Some(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broad_prefix_index_uses_static_routes_without_scanning_all_words() {
        let index = L2BroadPrefixIndex::build(&["проверка\nпроверяем\nработает\n"]);
        let candidates = index.prefix_candidates("пров", 4, 24, 8);

        assert_eq!(index.stats().source_words, 3);
        assert_eq!(candidates, vec!["проверка", "проверяем"]);
    }
}
