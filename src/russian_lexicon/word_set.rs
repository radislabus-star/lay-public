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
}
