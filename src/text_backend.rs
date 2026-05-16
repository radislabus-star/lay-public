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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextReplaceCapability {
    /// Best effort: key taps and backspaces can race with focused apps.
    KeyReplay,
    /// A whole tail can be deleted and committed as one backend call.
    AtomicTailReplace,
    /// A backend can address a minimal text range around the cursor.
    MinimalRangeReplace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextBackendCapabilities {
    pub preference: TextBackendPreference,
    pub replace: TextReplaceCapability,
    pub can_switch_layout: bool,
}

impl TextBackendCapabilities {
    pub const fn uinput() -> Self {
        Self {
            preference: TextBackendPreference::Uinput,
            replace: TextReplaceCapability::KeyReplay,
            can_switch_layout: true,
        }
    }

    pub const fn ime() -> Self {
        Self {
            preference: TextBackendPreference::Ime,
            replace: TextReplaceCapability::AtomicTailReplace,
            can_switch_layout: false,
        }
    }

    pub fn can_atomic_replace(self) -> bool {
        matches!(
            self.replace,
            TextReplaceCapability::AtomicTailReplace | TextReplaceCapability::MinimalRangeReplace
        )
    }
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

    #[test]
    fn exposes_backend_capabilities_for_decoder_policy() {
        assert!(!TextBackendCapabilities::uinput().can_atomic_replace());
        assert!(TextBackendCapabilities::uinput().can_switch_layout);
        assert!(TextBackendCapabilities::ime().can_atomic_replace());
        assert!(!TextBackendCapabilities::ime().can_switch_layout);
    }
}
