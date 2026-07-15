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
    /// Select IME when it owns the focused field. A dispatched edit is never
    /// retried through another backend.
    Ime,
    /// Select IME when focused, otherwise select uinput before dispatch.
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
        matches!(self, Self::Ime | Self::Auto)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImeReplaceRequest {
    pub backspaces: u32,
    pub text: String,
}

impl ImeReplaceRequest {
    pub fn committed_tail(original: &str, replacement: impl Into<String>) -> Self {
        let replacement = replacement.into();
        let backspaces = committed_tail_suffix_backspaces(original, &replacement);
        let text = committed_tail_suffix_text(original, &replacement);
        Self { backspaces, text }
    }

    pub fn is_noop(&self) -> bool {
        self.backspaces == 0 && self.text.is_empty()
    }
}

fn committed_tail_suffix_backspaces(original: &str, replacement: &str) -> u32 {
    let prefix = committed_tail_common_prefix_chars(original, replacement);
    original.chars().count().saturating_sub(prefix) as u32
}

fn committed_tail_suffix_text(original: &str, replacement: &str) -> String {
    let prefix = committed_tail_common_prefix_chars(original, replacement);
    replacement.chars().skip(prefix).collect()
}

fn committed_tail_common_prefix_chars(original: &str, replacement: &str) -> usize {
    original
        .chars()
        .zip(replacement.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

#[cfg(test)]
#[path = "text_backend_tests.rs"]
mod tests;
