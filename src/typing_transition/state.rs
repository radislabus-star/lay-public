use crate::correction_core::{TypingErrorClass, TypingErrorEvent};
use crate::text_metrics::{has_cyrillic, has_latin};
use crate::word_reader::{is_cyrillic_letters_only, last_text_word, split_edge_whitespace};
use crate::word_recognizer::is_ascii_technical_token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct L1SurfaceSignal {
    original: String,
    core: String,
    current_word: String,
    script: L1ScriptProfile,
}

impl L1SurfaceSignal {
    pub(crate) fn from_text(text: &str) -> Self {
        let (_, core, _) = split_edge_whitespace(text);
        let current_word = last_text_word(core).unwrap_or_default();
        let script = L1ScriptProfile::from_text(&current_word);

        Self {
            original: text.to_string(),
            core: core.to_string(),
            current_word,
            script,
        }
    }

    pub(crate) fn into_event(self) -> TypingErrorEvent {
        let input_class = self.classify_current_word();
        TypingErrorEvent {
            original: self.original,
            core: self.core,
            current_word: self.current_word,
            input_class,
        }
    }

    fn classify_current_word(&self) -> TypingErrorClass {
        if self.current_word.is_empty() {
            return TypingErrorClass::Unknown;
        }
        if is_ascii_technical_token(&self.current_word) {
            return TypingErrorClass::TechnicalToken;
        }
        if self.script == L1ScriptProfile::MixedCyrillicLatin {
            return TypingErrorClass::MixedScript;
        }
        if is_cyrillic_letters_only(&self.current_word)
            && !crate::russian_lexicon::is_known_russian_word_or_form(&self.current_word)
        {
            return TypingErrorClass::CompositeTypo;
        }
        if self.script == L1ScriptProfile::AsciiAlphabetic && self.current_word.chars().count() >= 3
        {
            return TypingErrorClass::WrongLayout;
        }
        TypingErrorClass::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L1ScriptProfile {
    Empty,
    Cyrillic,
    AsciiAlphabetic,
    MixedCyrillicLatin,
    Other,
}

impl L1ScriptProfile {
    fn from_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::Empty;
        }
        if has_cyrillic(text) && has_latin(text) {
            return Self::MixedCyrillicLatin;
        }
        if text.chars().all(crate::keyboard::is_cyrillic_letter) {
            return Self::Cyrillic;
        }
        if text.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Self::AsciiAlphabetic;
        }
        Self::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l1_surface_signal_preserves_current_event_shape() {
        let event = L1SurfaceSignal::from_text("  проверка ghbdtn ").into_event();

        assert_eq!(event.original, "  проверка ghbdtn ");
        assert_eq!(event.core, "проверка ghbdtn");
        assert_eq!(event.current_word, "ghbdtn");
        assert_eq!(event.input_class, TypingErrorClass::WrongLayout);
    }

    #[test]
    fn l1_surface_signal_detects_mixed_script_before_l2() {
        let event = L1SurfaceSignal::from_text("fавтозамена ").into_event();

        assert_eq!(event.current_word, "fавтозамена");
        assert_eq!(event.input_class, TypingErrorClass::MixedScript);
    }
}
