use crate::lexicon::{is_common_en_technical_word, is_common_ru_word};

pub const MAX_CONTEXT_TOKENS: usize = 32;
pub const MIN_CONTEXT_TOKENS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    CyrillicWord,
    AsciiWord,
    TechnicalAscii,
    Number,
    Punctuation,
    Mixed,
    Other,
}

impl TokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CyrillicWord => "cyrillic_word",
            Self::AsciiWord => "ascii_word",
            Self::TechnicalAscii => "technical_ascii",
            Self::Number => "number",
            Self::Punctuation => "punctuation",
            Self::Mixed => "mixed",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextToken {
    pub text: String,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailContext {
    pub tokens: Vec<ContextToken>,
}

impl TailContext {
    pub fn from_text(text: &str) -> Self {
        let mut tokens = text
            .split_whitespace()
            .map(|token| ContextToken {
                text: token.to_string(),
                kind: classify_token(token),
            })
            .collect::<Vec<_>>();
        if tokens.len() > MAX_CONTEXT_TOKENS {
            tokens = tokens[tokens.len() - MAX_CONTEXT_TOKENS..].to_vec();
        }
        Self { tokens }
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    pub fn has_min_phrase(&self) -> bool {
        self.tokens.len() >= MIN_CONTEXT_TOKENS.min(2)
    }

    pub fn previous(&self) -> Option<&ContextToken> {
        self.tokens
            .len()
            .checked_sub(2)
            .and_then(|idx| self.tokens.get(idx))
    }

    pub fn last(&self) -> Option<&ContextToken> {
        self.tokens.last()
    }

    pub fn has_technical_context(&self) -> bool {
        self.tokens.iter().any(|token| {
            token.kind == TokenKind::TechnicalAscii
                || token.text.starts_with('-')
                || token.text.contains('/')
                || token.text.contains('=')
                || token.text.contains("://")
        })
    }

    pub fn mixed_language_score(&self) -> f32 {
        let has_ru = self
            .tokens
            .iter()
            .any(|token| token.kind == TokenKind::CyrillicWord);
        let has_en = self.tokens.iter().any(|token| {
            token.kind == TokenKind::AsciiWord || token.kind == TokenKind::TechnicalAscii
        });
        if has_ru && has_en {
            0.25
        } else {
            0.0
        }
    }

    pub fn phrase_signature(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.kind.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn classify_token(token: &str) -> TokenKind {
    let trimmed = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    if trimmed.is_empty() && token.chars().all(|ch| ch.is_ascii_punctuation()) {
        return TokenKind::Punctuation;
    }
    if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return TokenKind::Number;
    }
    if trimmed.chars().all(|ch| ch.is_ascii_alphabetic()) {
        if is_common_en_technical_word(&trimmed.to_ascii_lowercase()) {
            return TokenKind::TechnicalAscii;
        }
        return TokenKind::AsciiWord;
    }
    if trimmed.chars().all(is_cyrillic_letter) {
        let lower = trimmed.to_lowercase();
        if is_common_ru_word(&lower) || lower.chars().count() <= 3 {
            return TokenKind::CyrillicWord;
        }
        return TokenKind::CyrillicWord;
    }
    if trimmed.chars().any(is_cyrillic_letter) && trimmed.chars().any(|ch| ch.is_ascii_alphabetic())
    {
        return TokenKind::Mixed;
    }
    TokenKind::Other
}

fn is_cyrillic_letter(ch: char) -> bool {
    ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch) || ch == 'ё' || ch == 'Ё'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_last_context_tokens() {
        let text = (0..40)
            .map(|idx| format!("w{idx}"))
            .collect::<Vec<_>>()
            .join(" ");
        let context = TailContext::from_text(&text);
        assert_eq!(context.token_count(), MAX_CONTEXT_TOKENS);
        assert_eq!(context.tokens.first().unwrap().text, "w8");
    }

    #[test]
    fn classifies_mixed_tail() {
        let context = TailContext::from_text("html вот api");
        assert_eq!(context.tokens[0].kind, TokenKind::TechnicalAscii);
        assert_eq!(context.tokens[1].kind, TokenKind::CyrillicWord);
        assert_eq!(context.tokens[2].kind, TokenKind::TechnicalAscii);
        assert!(context.mixed_language_score() > 0.0);
    }
}
