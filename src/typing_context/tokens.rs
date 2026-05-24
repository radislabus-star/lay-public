use crate::word_recognizer::{recognize_token, WordKind, WordScript};

pub(super) fn is_russian_context_token(token: &str) -> bool {
    let identity = recognize_token(token);
    matches!(
        identity.kind,
        WordKind::PlainWord | WordKind::TechnicalToken
    ) && identity.script == WordScript::Cyrillic
        && !identity.technical
}

pub(super) fn is_natural_english_context_token(token: &str) -> bool {
    if token_has_context_punctuation(token) {
        return false;
    }
    let identity = recognize_token(token);
    identity.kind == WordKind::PlainWord
        && identity.script == WordScript::Ascii
        && identity.known_en
        && !identity.technical
        && !identity.protected
}

pub(super) fn is_embedded_ascii_term_context_token(token: &str) -> bool {
    if token_has_context_punctuation(token) {
        return false;
    }
    let identity = recognize_token(token);
    matches!(
        identity.kind,
        WordKind::PlainWord | WordKind::TechnicalToken
    ) && identity.script == WordScript::Ascii
        && (identity.known_en || identity.technical || identity.protected)
}

pub(super) fn is_ascii_technical_context_token(token: &str) -> bool {
    if token_has_context_punctuation(token) {
        return false;
    }
    let identity = recognize_token(token);
    matches!(
        identity.kind,
        WordKind::PlainWord | WordKind::TechnicalToken
    ) && identity.script == WordScript::Ascii
        && (identity.technical || identity.protected)
}

pub(super) fn has_recent_russian_context_before_last(tokens_before_last: &[&str]) -> bool {
    tokens_before_last
        .iter()
        .rev()
        .skip(1)
        .take(4)
        .any(|token| is_russian_context_token(token))
}

fn token_has_context_punctuation(token: &str) -> bool {
    token.chars().any(|ch| {
        matches!(
            ch,
            '\'' | ';'
                | '['
                | ']'
                | '`'
                | ','
                | '.'
                | '?'
                | '!'
                | ':'
                | '$'
                | '%'
                | '^'
                | '&'
                | '|'
                | '#'
                | '@'
                | '/'
                | '\\'
        )
    })
}
