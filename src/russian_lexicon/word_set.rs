use std::collections::HashSet;

pub struct WordSet {
    words: Box<[String]>,
}

impl WordSet {
    pub(crate) fn from_words(words: HashSet<String>) -> Self {
        let mut words: Vec<String> = words.into_iter().collect();
        words.sort_unstable();
        words.dedup();
        Self {
            words: words.into_boxed_slice(),
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words
            .binary_search_by(|candidate| candidate.as_str().cmp(word))
            .is_ok()
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.words.iter()
    }

    pub(crate) fn prefix_words(
        &self,
        prefix: &str,
        min_chars: usize,
        max_chars: usize,
        limit: usize,
    ) -> Vec<String> {
        if prefix.is_empty() || limit == 0 {
            return Vec::new();
        }
        let start = self.words.partition_point(|word| word.as_str() < prefix);
        let mut out = Vec::with_capacity(limit.min(32));
        for word in self.words.iter().skip(start) {
            if !word.starts_with(prefix) {
                break;
            }
            let len = word.chars().count();
            if (min_chars..=max_chars).contains(&len) {
                out.push(word.clone());
                if out.len() >= limit {
                    break;
                }
            }
        }
        out
    }
}
