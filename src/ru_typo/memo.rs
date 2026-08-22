use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const WORD_MATERIAL_CACHE_CAPACITY: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WordMaterialKind {
    Plausible,
    VerbEnding,
    HardSign,
    RepeatedLetter,
    AdjacentTransposition,
    MissingLetter,
    SingleLetterSubstitution,
    VowelConfusion,
    ContextualVowelConfusion,
    ExtraLetters,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WordMaterialValue {
    Text(Option<String>),
    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WordMaterialEntry {
    generation: u64,
    kind: WordMaterialKind,
    input: String,
    value: WordMaterialValue,
}

struct WordMaterialCache {
    capacity: usize,
    entries: VecDeque<WordMaterialEntry>,
}

impl WordMaterialCache {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            capacity,
            entries: VecDeque::new(),
        }
    }

    fn text(
        &mut self,
        generation: u64,
        kind: WordMaterialKind,
        input: &str,
    ) -> Option<Option<String>> {
        let index = self.entries.iter().position(|entry| {
            entry.generation == generation && entry.kind == kind && entry.input == input
        })?;
        let entry = self.entries.remove(index)?;
        let WordMaterialValue::Text(value) = &entry.value else {
            return None;
        };
        let value = value.clone();
        self.entries.push_back(entry);
        Some(value)
    }

    fn boolean(&mut self, generation: u64, kind: WordMaterialKind, input: &str) -> Option<bool> {
        let index = self.entries.iter().position(|entry| {
            entry.generation == generation && entry.kind == kind && entry.input == input
        })?;
        let entry = self.entries.remove(index)?;
        let WordMaterialValue::Boolean(value) = entry.value else {
            return None;
        };
        self.entries.push_back(entry);
        Some(value)
    }

    fn insert(
        &mut self,
        generation: u64,
        kind: WordMaterialKind,
        input: &str,
        value: WordMaterialValue,
    ) {
        self.entries.retain(|entry| entry.generation == generation);
        if let Some(index) = self.entries.iter().position(|entry| {
            entry.generation == generation && entry.kind == kind && entry.input == input
        }) {
            self.entries.remove(index);
        }
        self.entries.push_back(WordMaterialEntry {
            generation,
            kind,
            input: input.to_string(),
            value,
        });
        while self.entries.len() > self.capacity {
            self.entries.pop_front();
        }
    }
}

fn word_material_cache() -> &'static Mutex<WordMaterialCache> {
    static CACHE: OnceLock<Mutex<WordMaterialCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(WordMaterialCache::new(WORD_MATERIAL_CACHE_CAPACITY)))
}

fn cacheable_word_material_input(input: &str) -> bool {
    crate::word_reader::is_cyrillic_word(input)
}

pub(super) fn memoized_text(
    kind: WordMaterialKind,
    input: &str,
    compute: impl FnOnce() -> Option<String>,
) -> Option<String> {
    if !cacheable_word_material_input(input) {
        return compute();
    }
    let generation = crate::nanda_wave::l2_field::candidate_material_generation();
    if let Some(value) = word_material_cache()
        .lock()
        .ok()
        .and_then(|mut cache| cache.text(generation, kind, input))
    {
        return value;
    }
    let value = compute();
    if crate::nanda_wave::l2_field::candidate_material_generation() != generation {
        return None;
    }
    if let Ok(mut cache) = word_material_cache().lock() {
        cache.insert(
            generation,
            kind,
            input,
            WordMaterialValue::Text(value.clone()),
        );
    }
    value
}

pub(super) fn memoized_bool(
    kind: WordMaterialKind,
    input: &str,
    compute: impl FnOnce() -> bool,
) -> bool {
    if !cacheable_word_material_input(input) {
        return compute();
    }
    let generation = crate::nanda_wave::l2_field::candidate_material_generation();
    if let Some(value) = word_material_cache()
        .lock()
        .ok()
        .and_then(|mut cache| cache.boolean(generation, kind, input))
    {
        return value;
    }
    let value = compute();
    if crate::nanda_wave::l2_field::candidate_material_generation() != generation {
        return false;
    }
    if let Ok(mut cache) = word_material_cache().lock() {
        cache.insert(generation, kind, input, WordMaterialValue::Boolean(value));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_text_material_is_cached_and_lru_is_bounded() {
        let mut cache = WordMaterialCache::new(2);
        cache.insert(
            7,
            WordMaterialKind::MissingLetter,
            "один",
            WordMaterialValue::Text(None),
        );
        assert_eq!(
            cache.text(7, WordMaterialKind::MissingLetter, "один"),
            Some(None)
        );
        cache.insert(
            7,
            WordMaterialKind::ExtraLetters,
            "два",
            WordMaterialValue::Text(Some("два".to_string())),
        );
        cache.insert(
            7,
            WordMaterialKind::RepeatedLetter,
            "три",
            WordMaterialValue::Text(None),
        );
        assert_eq!(cache.entries.len(), 2);
        assert!(cache
            .text(7, WordMaterialKind::MissingLetter, "один")
            .is_none());
    }

    #[test]
    fn generation_change_discards_old_material() {
        let mut cache = WordMaterialCache::new(4);
        cache.insert(
            3,
            WordMaterialKind::Plausible,
            "форма",
            WordMaterialValue::Boolean(true),
        );
        cache.insert(
            4,
            WordMaterialKind::Plausible,
            "форма",
            WordMaterialValue::Boolean(false),
        );
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(
            cache.boolean(4, WordMaterialKind::Plausible, "форма"),
            Some(false)
        );
        assert_eq!(cache.boolean(3, WordMaterialKind::Plausible, "форма"), None);
    }

    #[test]
    fn only_single_cyrillic_tokens_enter_word_material_cache() {
        assert!(cacheable_word_material_input("форма"));
        assert!(cacheable_word_material_input("Форма"));
        assert!(cacheable_word_material_input("форма-слово"));
        assert!(!cacheable_word_material_input("две формы"));
        assert!(!cacheable_word_material_input("forma"));
    }
}
