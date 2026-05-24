//! Token-level language recognition for layout arbiters.
//!
//! This is a small lexical layer: normalize a token, decide whether it is known
//! Russian/English, and evaluate whole-token sequences. It does not correct or
//! emit text.

use crate::word_reader::split_ws_segments;
use crate::word_recognizer::{recognize_token, WordKind, WordScript};

#[derive(Clone, Copy)]
pub(crate) enum Lang {
    Ru,
    En,
}

pub(crate) fn warm_up() {
    crate::word_recognizer::warm_up();
}

pub(crate) fn is_known_ru_token(token: &str) -> bool {
    let identity = recognize_token(token);
    identity.known_ru
        && identity.script == WordScript::Cyrillic
        && matches!(identity.kind, WordKind::PlainWord)
}

pub(crate) fn is_known_en_token(token: &str) -> bool {
    let identity = recognize_token(token);
    identity.script == WordScript::Ascii
        && (identity.known_en || identity.protected)
        && matches!(
            identity.kind,
            WordKind::PlainWord | WordKind::TechnicalToken
        )
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
            Lang::Ru => {
                identity.known_ru
                    && identity.script == WordScript::Cyrillic
                    && matches!(identity.kind, WordKind::PlainWord)
            }
            Lang::En => {
                identity.script == WordScript::Ascii
                    && (identity.known_en || identity.protected)
                    && matches!(
                        identity.kind,
                        WordKind::PlainWord | WordKind::TechnicalToken
                    )
            }
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
