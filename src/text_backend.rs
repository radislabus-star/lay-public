//! Text-edit backend selection.
//!
//! The correction engine decides *what* text should appear. This module only
//! describes *how* the daemon may apply that edit to the focused application.
//! The production backend is still uinput replay. The IME backend is an
//! opt-in path for an IBus/Fcitx-like engine that can delete surrounding text
//! and commit a replacement string without synthesizing keystrokes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBackendPreference {
    /// Keep the proven uinput path. This is the safe public default.
    Uinput,
    /// Try an IME bridge first, then let the daemon fall back if unavailable.
    Ime,
    /// Reserved for a future policy that can auto-detect a healthy IME engine.
    Auto,
}

impl TextBackendPreference {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "ime" | "ibus" | "input-method" => Self::Ime,
            "auto" => Self::Auto,
            _ => Self::Uinput,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uinput => "uinput",
            Self::Ime => "ime",
            Self::Auto => "auto",
        }
    }

    pub fn should_try_ime(self) -> bool {
        matches!(self, Self::Ime)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImeReplaceRequest {
    pub backspaces: u32,
    pub text: String,
}

impl ImeReplaceRequest {
    pub fn committed_tail(original: &str, replacement: impl Into<String>) -> Self {
        Self {
            backspaces: original.chars().count() as u32,
            text: replacement.into(),
        }
    }

    pub fn is_noop(&self) -> bool {
        self.backspaces == 0 && self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_backend_preference() {
        assert_eq!(
            TextBackendPreference::parse("uinput"),
            TextBackendPreference::Uinput
        );
        assert_eq!(
            TextBackendPreference::parse("ime"),
            TextBackendPreference::Ime
        );
        assert_eq!(
            TextBackendPreference::parse("IBUS"),
            TextBackendPreference::Ime
        );
        assert_eq!(
            TextBackendPreference::parse("auto"),
            TextBackendPreference::Auto
        );
        assert_eq!(
            TextBackendPreference::parse("unknown"),
            TextBackendPreference::Uinput
        );
    }

    #[test]
    fn ime_request_counts_unicode_tail_chars() {
        let request = ImeReplaceRequest::committed_tail("привет ", "hello ");
        assert_eq!(request.backspaces, 7);
        assert_eq!(request.text, "hello ");
        assert!(!request.is_noop());
    }
}
