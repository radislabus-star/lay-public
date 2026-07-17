//! Token-level language recognition for layout arbiters.
//!
//! This is a small lexical layer: normalize a token, decide whether it is known
//! Russian/English, and evaluate whole-token sequences. It does not correct or
//! emit text.

use crate::word_reader::split_ws_segments;
use crate::word_recognizer::recognize_token;

#[derive(Clone, Copy)]
pub(crate) enum Lang {
    Ru,
    En,
}

pub(crate) fn is_known_ru_token(token: &str) -> bool {
    recognize_token(token).is_known_russian_plain_word()
}

pub(crate) fn is_known_en_token(token: &str) -> bool {
    recognize_token(token).is_known_ascii_or_protected_token()
}

pub(crate) fn all_tokens_known(text: &str, lang: Lang) -> bool {
    let mut found = false;
    for (segment, is_ws) in split_ws_segments(text) {
        if is_ws {
            continue;
        }
        let identity = recognize_token(segment);
        if identity.core.is_empty() {
            return false;
        }
        found = true;
        let known = match lang {
            Lang::Ru => identity.is_known_russian_plain_word(),
            Lang::En => identity.is_known_ascii_or_protected_token(),
        };
        if !known {
            return false;
        }
    }
    found
}

#[cfg(test)]
#[path = "token_language_tests.rs"]
mod tests;
