//! ASCII-to-Cyrillic layout autoswitch.

mod candidate;
mod conjunction;
mod phrase;
mod preposition;
mod punctuation;
mod symbols;
mod word;

pub(crate) use conjunction::correct_contextual_ascii_conjunction_i;
pub(crate) use phrase::{correct_wrong_layout_ascii_phrase, is_confident_wrong_layout_ascii_pair};
pub(crate) use preposition::correct_contextual_ascii_preposition_v;
pub(crate) use symbols::{
    ascii_layout_prefix_can_be_letter, is_ascii_layout_letter_surface,
    is_ascii_layout_letter_symbol, is_protected_ascii_layout_token,
};
pub(crate) use word::{
    correct_confident_wrong_layout_ascii_word, correct_wrong_layout_ascii_word,
    correct_wrong_layout_ascii_word_experimental,
};
