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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LatentTypingState {
    pub(crate) text: String,
    pub(crate) context_words: Vec<String>,
    pub(crate) current_word: String,
    pub(crate) word_count: usize,
    pub(crate) script: LatentScriptProfile,
    pub(crate) current_word_known: bool,
}

impl LatentTypingState {
    pub(crate) fn from_text(text: &str) -> Self {
        let words = normalized_words(text);
        let current_word = words.last().cloned().unwrap_or_default();
        let context_words = words
            .get(..words.len().saturating_sub(1))
            .unwrap_or_default()
            .to_vec();
        let script = LatentScriptProfile::from_text(&current_word);
        let current_word_known = current_word_is_known(&current_word);

        Self {
            text: text.to_string(),
            context_words,
            current_word,
            word_count: words.len(),
            script,
            current_word_known,
        }
    }

    pub(crate) fn context_changed(&self, other: &Self) -> bool {
        self.context_words != other.context_words
    }

    pub(crate) fn word_count_changed(&self, other: &Self) -> bool {
        self.word_count != other.word_count
    }

    pub(crate) fn current_word_changed(&self, other: &Self) -> bool {
        self.current_word != other.current_word
    }

    pub(crate) fn known_word_drift_to(&self, other: &Self) -> bool {
        self.current_word_known
            && other.current_word_known
            && self.current_word_changed(other)
            && self.script == other.script
            && matches!(
                self.script,
                LatentScriptProfile::Cyrillic | LatentScriptProfile::AsciiAlphabetic
            )
    }

    pub(crate) fn candidate_imported_left_context(&self, other: &Self) -> bool {
        self.word_count == other.word_count
            && self.context_changed(other)
            && self.current_word_changed(other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LatentScriptProfile {
    Empty,
    Cyrillic,
    AsciiAlphabetic,
    MixedCyrillicLatin,
    Other,
}

impl LatentScriptProfile {
    fn from_text(text: &str) -> Self {
        match L1ScriptProfile::from_text(text) {
            L1ScriptProfile::Empty => Self::Empty,
            L1ScriptProfile::Cyrillic => Self::Cyrillic,
            L1ScriptProfile::AsciiAlphabetic => Self::AsciiAlphabetic,
            L1ScriptProfile::MixedCyrillicLatin => Self::MixedCyrillicLatin,
            L1ScriptProfile::Other => Self::Other,
        }
    }
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let (_, word, _) = crate::word_reader::split_word_punctuation(token);
            (!word.is_empty()).then(|| word.to_lowercase())
        })
        .collect()
}

fn current_word_is_known(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let lower = word.to_lowercase();
    crate::russian_lexicon::is_known_russian_word_or_form(&lower)
        || word_has_common_usage_authority(&lower)
        || is_ascii_technical_token(word)
}

pub(crate) fn word_has_common_usage_authority(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let lower = word.to_lowercase();
    crate::lexicon::is_common_ru_word(&lower)
        || usage_snapshot_has_word_authority(
            &crate::nanda_wave::cached_usage_prior_snapshot(),
            &lower,
        )
}

fn usage_snapshot_has_word_authority(
    usage: &crate::nanda_wave::UsagePriorSnapshot,
    word: &str,
) -> bool {
    let readout = usage.hot_readout(&[], "*", "*", "*", word);
    readout.accepted_count >= 2 && readout.accepted_count > readout.rejected_count
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

    #[test]
    fn latent_state_separates_context_from_current_word() {
        let before = LatentTypingState::from_text("мы можем ");
        let after = LatentTypingState::from_text("мы модем ");

        assert_eq!(before.context_words, ["мы"]);
        assert_eq!(before.current_word, "можем");
        assert_eq!(after.current_word, "модем");
        assert!(before.known_word_drift_to(&after));
        assert!(!before.candidate_imported_left_context(&after));
    }

    #[test]
    fn latent_state_detects_context_tainted_candidate() {
        let before = LatentTypingState::from_text("можем ");
        let after = LatentTypingState::from_text("мы модем ");

        assert!(before.context_changed(&after));
        assert!(before.word_count_changed(&after));
    }

    #[test]
    fn raw_repeated_typo_does_not_become_known_word_authority() {
        let usage = crate::nanda_wave::usage_prior::snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"typed","word":"деллай","context":[]}
{"ts":2,"kind":"typed","word":"деллай","context":[]}
{"ts":3,"kind":"typed","word":"деллай","context":[]}
"#,
        );

        assert!(!usage_snapshot_has_word_authority(&usage, "деллай"));
    }

    #[test]
    fn repeated_accepted_use_can_become_known_word_authority() {
        let usage = crate::nanda_wave::usage_prior::snapshot_from_usage_events_for_tests(
            r#"{"ts":1,"kind":"accepted_ime","word":"кастомтерм","context":[]}
{"ts":2,"kind":"accepted_ime","word":"кастомтерм","context":[]}
"#,
        );

        assert!(usage_snapshot_has_word_authority(&usage, "кастомтерм"));
    }
}
