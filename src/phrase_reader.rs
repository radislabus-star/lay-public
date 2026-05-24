//! Phrase-level reading and scoring for typing assist.
//!
//! This facade exposes corrections that need more than one token of context:
//! glued words, accidentally split words, and a letter moved into the next
//! word. Runtime output and daemon state live elsewhere.

#[path = "phrase_reader/contextual_tail.rs"]
mod contextual_tail;
#[path = "phrase_reader/glued_phrase.rs"]
mod glued_phrase;
#[path = "phrase_reader/guards.rs"]
mod guards;
#[path = "phrase_reader/moved_prefix.rs"]
mod moved_prefix;
#[path = "phrase_reader/split_pair.rs"]
mod split_pair;

pub use contextual_tail::correct_contextual_glued_tail;
pub use glued_phrase::correct_glued_russian_phrase;
pub use moved_prefix::correct_moved_prefix_letter_pair;
pub use split_pair::correct_split_word_pair;

#[cfg(test)]
#[path = "phrase_reader_tests.rs"]
mod tests;
