use crate::word_recognizer::{recognize_token, WordIdentity, WordScript};

pub(super) fn is_russian_context_token(token: &str) -> bool {
    ContextToken::new(token).is_russian_context()
}

pub(super) fn is_natural_english_context_token(token: &str) -> bool {
    ContextToken::new(token).is_natural_english()
}

pub(super) fn is_embedded_ascii_term_context_token(token: &str) -> bool {
    ContextToken::new(token).is_embedded_ascii_term()
}

pub(super) fn is_ascii_technical_context_token(token: &str) -> bool {
    ContextToken::new(token).is_ascii_technical()
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

#[derive(Debug, Clone, Copy)]
struct ContextToken<'a> {
    identity: WordIdentity<'a>,
    has_context_punctuation: bool,
}

impl<'a> ContextToken<'a> {
    fn new(token: &'a str) -> Self {
        Self {
            identity: recognize_token(token),
            has_context_punctuation: token_has_context_punctuation(token),
        }
    }

    fn is_russian_context(self) -> bool {
        self.identity.is_plain_or_technical()
            && self.identity.script == WordScript::Cyrillic
            && !self.identity.technical
    }

    fn is_natural_english(self) -> bool {
        !self.has_context_punctuation
            && self.identity.is_plain_word()
            && self.identity.script == WordScript::Ascii
            && self.identity.known_en
            && !self.identity.technical
            && !self.identity.protected
    }

    fn is_embedded_ascii_term(self) -> bool {
        !self.has_context_punctuation
            && self.identity.is_plain_or_technical()
            && self.identity.script == WordScript::Ascii
            && (self.identity.known_en || self.identity.technical || self.identity.protected)
    }

    fn is_ascii_technical(self) -> bool {
        !self.has_context_punctuation
            && self.identity.is_plain_or_technical()
            && self.identity.script == WordScript::Ascii
            && (self.identity.technical || self.identity.protected)
    }
}
