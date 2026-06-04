//! English lexical helpers for layout autoswitch.

use crate::lexicon::is_common_en_technical_word;
use crate::word_recognizer::{recognize_token, WordScript};

pub(crate) fn is_known_english_layout_autoswitch_word(word: &str) -> bool {
    if is_common_en_technical_word(&word.to_ascii_lowercase()) {
        return true;
    }
    let identity = recognize_token(word);
    identity.script == WordScript::Ascii && identity.known_en
}

pub(super) fn is_plain_ascii_word_candidate(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}
