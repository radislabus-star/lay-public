use super::{
    normalize_typing_assist_pipeline, CorrectionEngine, CorrectionSafety, LayConfig,
    TypingAssistRuleConfig,
};
use crate::desktop::{resolve_layout_backend, LayoutBackend};
use crate::text_backend::TextBackendPreference;

impl LayConfig {
    pub fn active_replace_words(&self) -> usize {
        self.replace_words.clamp(1, 3)
    }

    pub fn active_typing_assist_words(&self) -> usize {
        self.typing_assist_words.clamp(1, 3)
    }

    pub fn active_multi_tap_max_taps(&self) -> u8 {
        self.multi_tap_max_taps.clamp(2, 4)
    }

    pub fn active_correction_engine(&self) -> CorrectionEngine {
        match self.correction_engine.as_deref() {
            Some("smart") => CorrectionEngine::Smart,
            Some("replay") => CorrectionEngine::Replay,
            _ if self.mode == "llm" => CorrectionEngine::Smart,
            _ => CorrectionEngine::Replay,
        }
    }

    pub fn active_layout_backend(&self) -> LayoutBackend {
        resolve_layout_backend(
            &self.layout_backend,
            std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref(),
            std::env::var("DESKTOP_SESSION").ok().as_deref(),
            std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        )
    }

    pub fn active_text_backend(&self) -> TextBackendPreference {
        TextBackendPreference::parse(&self.text_backend)
    }

    pub fn active_typing_assist_pipeline(&self) -> Vec<TypingAssistRuleConfig> {
        normalize_typing_assist_pipeline(&self.typing_assist_pipeline)
    }

    pub fn active_correction_safety(&self) -> CorrectionSafety {
        match self.correction_safety.trim().to_ascii_lowercase().as_str() {
            "strict" | "safe" | "ultra-safe" | "ultrasafe" => CorrectionSafety::Strict,
            "experimental" | "exp" => CorrectionSafety::Experimental,
            _ => CorrectionSafety::Normal,
        }
    }

    pub fn lem_enabled_for_scope(&self, word_count: usize) -> bool {
        if !self.lem_enabled || self.active_lem_weight() <= 0.0 {
            return false;
        }
        word_count >= 2
    }
}
