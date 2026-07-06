use std::collections::{BTreeMap, HashMap, HashSet};

use crate::keyboard::is_cyrillic_letter;

pub(crate) fn balanced_l2_surface_words<I>(source: I, limit: usize) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    balanced_l2_words_by(source, limit, normalize_l2_surface_word)
}

fn balanced_l2_words_by<I>(
    source: I,
    limit: usize,
    normalize: fn(&str) -> Option<String>,
) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    if limit == 0 {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let words = source
        .into_iter()
        .filter_map(|word| normalize(&word))
        .filter(|word| seen.insert(word.clone()))
        .collect::<Vec<_>>();
    if words.len() <= limit {
        return words;
    }

    let mut words = words
        .into_iter()
        .map(|word| RankedSurfaceWord {
            usage: usage_priority(&word),
            common: crate::lexicon::is_common_ru_word(&word),
            len: word.chars().count(),
            word,
        })
        .collect::<Vec<_>>();
    words.sort_by(|left, right| {
        right
            .usage
            .cmp(&left.usage)
            .then_with(|| right.common.cmp(&left.common))
            .then_with(|| left.len.cmp(&right.len))
            .then_with(|| left.word.cmp(&right.word))
    });

    let mut selected = Vec::with_capacity(limit);
    let mut selected_set = HashSet::new();
    for item in words.iter().filter(|item| item.usage > 0) {
        if selected.len() >= limit / 2 {
            break;
        }
        if selected_set.insert(item.word.clone()) {
            selected.push(item.word.clone());
        }
    }
    for item in words.iter().filter(|item| item.common) {
        if selected.len() >= limit {
            return selected;
        }
        if selected_set.insert(item.word.clone()) {
            selected.push(item.word.clone());
        }
    }

    let mut buckets = BTreeMap::<char, Vec<String>>::new();
    for item in words {
        if selected_set.contains(&item.word) {
            continue;
        }
        let Some(first) = item.word.chars().next() else {
            continue;
        };
        buckets.entry(first).or_default().push(item.word);
    }
    let keys = buckets.keys().copied().collect::<Vec<_>>();
    let mut positions = HashMap::<char, usize>::new();
    while selected.len() < limit {
        let mut advanced = false;
        for key in &keys {
            if selected.len() >= limit {
                break;
            }
            let pos = positions.entry(*key).or_default();
            let Some(bucket) = buckets.get(key) else {
                continue;
            };
            let Some(word) = bucket.get(*pos) else {
                continue;
            };
            *pos += 1;
            advanced = true;
            if selected_set.insert(word.clone()) {
                selected.push(word.clone());
            }
        }
        if !advanced {
            break;
        }
    }
    selected
}

#[derive(Debug)]
struct RankedSurfaceWord {
    word: String,
    usage: u16,
    common: bool,
    len: usize,
}

pub(crate) fn normalize_l2_surface_word(word: &str) -> Option<String> {
    let normalized = word.trim().to_lowercase();
    let len = normalized.chars().count();
    if !(4..=24).contains(&len) {
        return None;
    }
    if !normalized.chars().all(is_cyrillic_letter) {
        return None;
    }
    if crate::lexicon::is_ru_live_protected_word(&normalized) {
        return None;
    }
    Some(normalized)
}

pub(crate) fn normalize_l2_training_surface_word(word: &str) -> Option<String> {
    let normalized = word.trim().to_lowercase();
    let len = normalized.chars().count();
    if !(1..=24).contains(&len) {
        return None;
    }
    if !normalized.chars().all(is_cyrillic_letter) {
        return None;
    }
    if crate::lexicon::is_ru_live_protected_word(&normalized) {
        return None;
    }
    if len >= 4
        || crate::lexicon::is_ru_one_letter_function_word(&normalized)
        || crate::lexicon::is_ru_short_function_word(&normalized)
        || crate::lexicon::is_common_ru_word(&normalized)
    {
        Some(normalized)
    } else {
        None
    }
}

fn usage_priority(word: &str) -> u16 {
    super::usage_prior::accepted_word_usage_count_cached(word).min(u16::MAX as u32) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l2_surface_bank_rejects_fragments() {
        assert_eq!(normalize_l2_surface_word("ко"), None);
        assert_eq!(normalize_l2_surface_word("ка"), None);
        assert_eq!(normalize_l2_surface_word("гл"), None);
        assert_eq!(
            normalize_l2_surface_word("комитет").as_deref(),
            Some("комитет")
        );
    }

    #[test]
    fn l2_training_surface_bank_keeps_known_short_function_words() {
        assert_eq!(
            normalize_l2_training_surface_word("и").as_deref(),
            Some("и")
        );
        assert_eq!(
            normalize_l2_training_surface_word("в").as_deref(),
            Some("в")
        );
        assert_eq!(
            normalize_l2_training_surface_word("не").as_deref(),
            Some("не")
        );
        assert_eq!(normalize_l2_training_surface_word("гл"), None);
    }
}
